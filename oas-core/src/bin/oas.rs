use anyhow::anyhow;
use async_std::stream::StreamExt;
use async_std::task;
use clap::Clap;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, time};

use oas_common::types::AudioObject;
use oas_common::{mapping, Record, TypedValue};
use oas_core::couch::{Doc, DocMeta};
use oas_core::rss::Feed;
use oas_core::{celery, couch, elastic};

const DEFAULT_HOST: &str = "http://localhost:5984";
const DEFAULT_DB: &str = "oas_test2";

pub struct State {
    pub db: couch::CouchDB,
}

#[derive(Clap)]
struct Args {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Clap)]
enum Command {
    /// Watch and show print changes from the CouchDB feed
    Watch(WatchOpts),
    /// Print all docs from CouchDB
    List,
    Debug,
    /// Run the indexing pipeline
    Index,
    /// Fetch a RSS feed
    Feed(FeedOpts),
    /// Create a test task
    Task,
}

#[derive(Clap)]
struct WatchOpts {
    /// Rev to start the watch stream at.
    since: Option<String>,
}

#[derive(Clap)]
struct FeedOpts {
    /// Feed URL to ingest
    url: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let args = Args::parse();
    let couch_config = couch::Config {
        host: DEFAULT_HOST.to_string(),
        database: DEFAULT_DB.to_string(),
        user: Some("admin".to_string()),
        password: Some("password".to_string()),
    };
    let db = couch::CouchDB::with_config(couch_config)?;
    db.init().await?;
    let state = State { db };

    let result = match args.command {
        Command::Watch(opts) => run_watch(state, opts).await,
        Command::List => run_list(state).await,
        Command::Debug => run_debug(state).await,
        Command::Index => run_index(state).await,
        Command::Feed(opts) => run_feed(state, opts).await,
        Command::Task => run_task().await,
    };
    eprintln!("RESULT {:?}", result);
    result
}

async fn run_task() -> anyhow::Result<()> {
    celery::run_celery().await?;
    // faktory::run_faktory().await?;
    Ok(())
}

async fn run_list(state: State) -> anyhow::Result<()> {
    let db = state.db;
    let mut params = HashMap::new();
    params.insert("include_docs".into(), "true".into());
    let docs: couch::DocList = db
        .get("_all_docs", Some(params))
        .await
        .map_err(|e| anyhow::anyhow!(e))?;
    eprintln!("docs: {:?}", docs);
    eprintln!(
        "typed docs: {:#?}",
        docs.clone().into_typed_docs::<Record<AudioObject>>()
    );
    Ok(())
}

async fn run_debug(state: State) -> anyhow::Result<()> {
    let db = state.db;
    let iter = 10_000;
    // let iter = 3;
    let start = time::Instant::now();
    let per_batch = 10.min(iter);
    let mut batch = vec![];
    for i in 0..iter {
        let doc = AudioObject {
            headline: Some(format!("hello {}", i)),
            ..Default::default()
        };
        let id = format!("test{}", i);
        let meta = DocMeta::with_id(id.to_string());
        let doc = Doc::from_typed(meta, doc)?;
        batch.push(doc);
        if batch.len() >= per_batch {
            let res = db
                .put_bulk_update(batch.clone())
                .await
                .map_err(|e| anyhow!(e))?;
            eprintln!("res: {:?}", res);
            batch.clear();
        }
    }
    let elapsed = start.elapsed();
    eprintln!("took {:?}", elapsed);
    eprintln!("per second: {}", iter as f32 / elapsed.as_secs_f32());
    Ok(())
}

async fn run_watch(state: State, opts: WatchOpts) -> anyhow::Result<()> {
    let db = state.db;
    let mut stream = db.changes_stream(opts.since);
    stream.set_infinite(true);
    while let Some(event) = stream.next().await {
        let event = event?;
        if let Some(doc) = event.doc {
            let record = doc.into_typed_record::<AudioObject>();
            eprintln!("record: {:#?}", record);
        }
    }
    Ok(())
}

async fn run_index(state: State) -> anyhow::Result<()> {
    let db = state.db;
    let index_name = "rust-test-1".to_string();
    let client = elastic::create_client()?;
    elastic::ensure_index(&client, &index_name, true).await?;

    let start = time::Instant::now();

    let batch_size = 1000;
    let mut batch = vec![];
    let mut count = 0;
    let mut stream = db.changes_stream(None);
    // stream.set_infinite(true);
    while let Some(event) = stream.next().await {
        let event = event?;
        eprintln!("EVENT: {:?}", event);
        if let Some(doc) = event.doc {
            let record = doc.into_typed_record::<AudioObject>();
            eprintln!("record: {:?}", record);
            // let id = doc.id().to_string();
            // let doc = doc.into_typed::<NameDoc>()?;
            // let record = Record::from_id_and_value(id, doc);
            match record {
                Ok(record) => batch.push(record),
                Err(e) => eprintln!("failed to convert doc to Record<AudioObject>: {}", e),
            }

            if batch.len() == batch_size {
                let res = elastic::index_records(&client, &index_name, &batch).await?;
                eprintln!("INDEXED {}", count);
                eprintln!("RES {:?}", res);
                count += batch.len();
                batch.clear()
            }
            // eprintln!("doc: {:?}", doc.into_typed::<NameDoc>()?);
        }
    }

    if batch.len() > 0 {
        let res = elastic::index_records(&client, &index_name, &batch).await?;
        eprintln!("INDEXED {}", count);
        eprintln!("RES {:?}", res);
        count += batch.len();
        batch.clear()
    }

    // if batch.len() > 0 {
    //     let res = elastic::index_records(&client, &index_name, batch).await?;
    //     eprintln!("RES {:?}", res);
    //     count += batch.len();
    //     batch.clear()
    // }
    eprintln!("FINISHED indexing {} docs", count);
    let elapsed = start.elapsed();
    eprintln!("took {:?}", elapsed);
    eprintln!("per second: {}", count as f32 / elapsed.as_secs_f32());
    // use elasticsearch::*;
    // let client = Elasticsearch::default();
    // let op = BulkOperations
    //      let response = client
    //     .bulk(BulkParts::Index(POSTS_INDEX))
    //     .body(body)
    //     .send()
    //     .await?;

    // let mut stream = db.changes(opts.since);
    // while let Some(event) = stream.next().await {
    //     let event = event?;
    //     if let Some(doc) = event.doc {
    //         eprintln!("doc: {:?}", doc.into_typed::<NameDoc>()?);
    //     }
    // }
    Ok(())
}

async fn run_feed(state: State, opts: FeedOpts) -> anyhow::Result<()> {
    let mut feed = Feed::new(opts.url)?;
    feed.load().await?;
    let items = feed.into_audio_objects()?;
    eprintln!("ITEMS: {:#?}", items);
    let docs: Vec<Doc> = items
        .into_iter()
        .map(|item| Doc::from_typed_record(item))
        .collect();
    eprintln!("DOCS: {:#?}", docs);
    let db = state.db;
    let res = db.put_bulk_update(docs).await.map_err(|e| anyhow!(e))?;
    eprintln!("RES: {:?}", res);
    Ok(())
}
