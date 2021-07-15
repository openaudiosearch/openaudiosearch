use async_std::stream::StreamExt;
use clap::Clap;
use oas_common::reference::Reference;
use oas_common::types::Media;
use oas_common::util;
use oas_common::Record;
use oas_common::Resolvable;
use oas_common::TypedValue;
use oas_core::couch::PutResult;
use oas_core::rss;
use oas_core::server::{run_server, ServerOpts};
use oas_core::types::Post;
use oas_core::util::*;
use oas_core::State;
use oas_core::{couch, elastic, tasks};
use std::time;
use url::Url;

const COUCHDB_HOST: &str = "http://localhost:5984";
const COUCHDB_DATABASE: &str = "oas_test2";
const ELASTICSEARCH_INDEX: &str = "oas";

#[derive(Clap)]
struct Args {
    #[clap(subcommand)]
    command: Command,
    // Elastic config
    // elastic: elastic::Config,
    // CouchDB config
    // couchdb: couch::Config
}

#[derive(Clap)]
enum Command {
    /// Watch and show print changes from the CouchDB feed
    Watch(WatchOpts),
    /// Print all docs from CouchDB
    List(ListOpts),
    Debug,
    /// Run the indexing pipeline
    Index(IndexOpts),
    /// Search for records
    Search(SearchOpts),
    /// Fetch a RSS feed
    Feed(FeedCommands),
    /// Create a test task
    Task(tasks::TaskOpts),
    /// Run the HTTP API server
    Server(ServerOpts),
}

#[derive(Clap)]
struct FeedCommands {
    /// Subcommand
    #[clap(subcommand)]
    command: FeedCommand,
}

#[derive(Clap)]
enum FeedCommand {
    /// Fetch a feed by URL.
    Fetch(FeedFetchOpts),
    /// Fetch and crawl a feed by URL (increasing offset param).
    Crawl(rss::ops::CrawlOpts),
}

#[derive(Clap)]
struct FeedFetchOpts {
    /// Feed URL
    url: Url,
}

#[derive(Clap)]
struct IndexOpts {
    /// Run forever in daemon mode
    #[clap(short, long)]
    daemon: bool,

    /// Delete and recreate the index
    #[clap(long)]
    recreate: bool,
}

#[derive(Clap)]
struct WatchOpts {
    /// Rev to start the watch stream at.
    since: Option<String>,
}

// #[derive(Clap)]
// pub struct FeedOpts {
//     /// Feed URL to ingest
//     url: Url,
//     /// Crawl a paginated feed recursively.
//     #[clap(short, long)]
//     crawl: bool,

//     /// Max number of pages to crawl.
//     #[clap(long)]
//     crawl_max: Option<usize>,
// }

#[derive(Clap)]
struct SearchOpts {
    /// Search query
    query: String,
}

#[derive(Clap)]
struct ListOpts {
    /// ID prefix (type)
    prefix: Option<String>,
    // Output as JSON
    // #[clap(long)]
    // json: bool,
}

#[async_std::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let args = Args::parse();

    let couch_config = couch::Config {
        host: COUCHDB_HOST.to_string(),
        database: COUCHDB_DATABASE.to_string(),
        user: Some("admin".to_string()),
        password: Some("password".to_string()),
    };

    let elastic_config = elastic::Config::with_default_url(ELASTICSEARCH_INDEX.to_string());

    let db = couch::CouchDB::with_config(couch_config)?;
    let index = elastic::Index::with_config(elastic_config)?;

    let state = State { db, index };

    let result = match args.command {
        Command::Watch(opts) => run_watch(state, opts).await,
        Command::List(opts) => run_list(state, opts).await,
        Command::Debug => run_debug(state).await,
        Command::Index(opts) => run_index(state, opts).await,
        Command::Search(opts) => run_search(state, opts).await,
        Command::Feed(opts) => run_feed(state, opts.command).await,
        Command::Task(opts) => run_task(state, opts).await,
        Command::Server(opts) => run_server(state, opts).await,
    };
    result
}

async fn run_task(state: State, opts: tasks::TaskOpts) -> anyhow::Result<()> {
    tasks::run_celery(state, opts).await?;
    // faktory::run_faktory().await?;
    Ok(())
}

async fn run_list(state: State, opts: ListOpts) -> anyhow::Result<()> {
    let db = state.db;
    let docs = match opts.prefix {
        Some(prefix) => db.get_all_with_prefix(&prefix).await?,
        None => db.get_all().await?,
    };
    let len = docs.rows.len();
    for doc in docs.rows() {
        println!("{}", serde_json::to_string(&doc).unwrap());
        // eprintln!("{}", serde_json::to_string_pretty(&doc).unwrap());
        // match opts.json {
        //     true =>
        // }
    }
    log::info!("total {}", len);

    // let docs = db.get_all::<serde_json::Value>().await?;
    // eprintln!("docs {:#?}", &docs);
    // let mut params = HashMap::new();
    // params.insert("include_docs".into(), "true".into());
    // let docs: couch::DocList = db
    //     .get("_all_docs", Some(params))
    //     .await
    //     .map_err(|e| anyhow::anyhow!(e))?;
    // eprintln!("docs: {:?}", docs);
    // eprintln!(
    //     "typed docs: {:#?}",
    //     docs.clone().into_typed_docs::<Record<Media>>()
    // );
    Ok(())
}

async fn run_debug(state: State) -> anyhow::Result<()> {
    let db = state.db;
    let media1 = Media {
        content_url: "http://foo.bar/m1.mp3".to_string(),
        ..Default::default()
    };
    let media2 = Media {
        content_url: "http://foo.bar/m2.mp3".to_string(),
        duration: Some(300.),
        ..Default::default()
    };
    let media1 =
        Record::from_id_and_value(util::id_from_hashed_string(&media1.content_url), media1);
    let media2 =
        Record::from_id_and_value(util::id_from_hashed_string(&media2.content_url), media2);
    let medias = vec![media1, media2];
    let res = db.put_record_bulk_update(medias).await?;
    let media_refs = res
        .into_iter()
        .filter_map(|r| match r {
            PutResult::Ok(r) => Some(Reference::Id(r.id)),
            PutResult::Err(_err) => None,
        })
        .collect();
    let post = Post {
        headline: Some("Hello world".into()),
        // media: vec![Reference::Id("id1"), Reference::Id("id2")],
        media: media_refs,
        ..Default::default()
    };
    let post = Record::from_id_and_value("testpost".to_string(), post);
    let id = post.guid().to_string();
    let res = db.put_record(post).await?;
    eprintln!("put res {:#?}", res);

    let mut post = db.get_record::<Post>(&id).await?;
    eprintln!("get post {:#?}", post);
    post.value.resolve_refs(&db).await?;
    eprintln!("resolved post {:#?}", post);
    let extracted_refs = post.value.extract_refs();
    eprintln!("extracted refs {:#?}", extracted_refs);
    eprintln!("post after extract {:#?}", post);
    Ok(())
    // let db = state.db;
    // let iter = 10_000;
    // // let iter = 3;
    // let start = time::Instant::now();
    // let per_batch = 10.min(iter);
    // let mut batch = vec![];
    // for i in 0..iter {
    //     let doc = Media {
    //         headline: Some(format!("hello {}", i)),
    //         ..Default::default()
    //     };
    //     let id = format!("test{}", i);
    //     let meta = DocMeta::with_id(id.to_string());
    //     let doc = Doc::from_typed(meta, doc)?;
    //     batch.push(doc);
    //     if batch.len() >= per_batch {
    //         let res = db
    //             .put_bulk_update(batch.clone())
    //             .await
    //             .map_err(|e| anyhow!(e))?;
    //         eprintln!("res: {:?}", res);
    //         batch.clear();
    //     }
    // }
    // let elapsed = start.elapsed();
    // eprintln!("took {:?}", elapsed);
    // eprintln!("per second: {}", iter as f32 / elapsed.as_secs_f32());
    // Ok(())
}

async fn run_watch(state: State, opts: WatchOpts) -> anyhow::Result<()> {
    let db = state.db;
    let mut stream = db.changes(opts.since);
    stream.set_infinite(true);
    while let Some(event) = stream.next().await {
        let event = event?;
        if let Some(doc) = event.doc {
            let record = doc.into_typed_record::<Media>();
            eprintln!("record: {:#?}", record);
        }
    }
    Ok(())
}

async fn run_index(state: State, opts: IndexOpts) -> anyhow::Result<()> {
    let db = state.db;
    let index = state.index;

    index.ensure_index(opts.recreate).await?;

    let start = time::Instant::now();

    let batch_size = 1000;
    let mut batch = vec![];
    let mut total = 0;
    let mut stream = db.changes(None);
    stream.set_infinite(opts.daemon);
    while let Some(event) = stream.next().await {
        let event = event?;
        if let Some(doc) = event.doc {
            let is_first_rev = doc.is_first_rev();
            // eprintln!(
            //     "incoming couch doc: {} (first? {:?})",
            //     doc.id(),
            //     is_first_rev
            // );
            let _id = doc.id().to_string();
            let record = doc.into_untyped_record();
            match record {
                Err(_err) => {}
                Ok(record) => match record.typ() {
                    Media::NAME => {
                        match is_first_rev {
                            Some(true) | None => {
                                // Do nothing for first revs of media recors.
                            }
                            Some(false) => {
                                match index.update_nested_record("media", &record).await {
                                    Err(e) => log::debug!("{}", e),
                                    Ok(_) => {}
                                };
                            }
                        }
                    }
                    Post::NAME => {
                        let record = record.into_typed_record::<Post>();
                        match record {
                            Ok(mut record) => {
                                let _res = record.resolve_refs(&db).await;
                                batch.push(record);
                            }
                            Err(e) => log::debug!("{}", e),
                        }
                    }
                    _ => {}
                },
            }

            if batch.len() == batch_size {
                let _res = index.put_typed_records(&batch).await?;
                total += batch.len();
                batch.clear()
            }
        }
    }

    if !batch.is_empty() {
        eprintln!("BATCH {:#?}", batch);
        let _res = index.put_typed_records(&batch).await?;
        total += batch.len();
        batch.clear()
    }

    let elapsed = start.elapsed().as_secs_f32();
    log::info!(
        "indexed {} records in {}s ({}/s)",
        total,
        elapsed,
        total as f32 / elapsed
    );
    Ok(())
}

async fn run_search(state: State, opts: SearchOpts) -> anyhow::Result<()> {
    let index = state.index;
    let records = index.find_records_with_text_query(&opts.query).await?;
    eprintln!("res: {:?}", records);
    let records: Vec<Record<Post>> = records
        .into_iter()
        .filter_map(|r| r.into_typed_record::<Post>().ok())
        .collect();
    debug_print_records(&records[..]);
    Ok(())
}

async fn run_feed(state: State, command: FeedCommand) -> anyhow::Result<()> {
    state.db.init().await?;
    match command {
        FeedCommand::Fetch(opts) => {
            rss::ops::fetch_and_save(&state.db, &opts.url).await?;
        }
        FeedCommand::Crawl(opts) => {
            rss::ops::crawl_and_save(&state.db, &opts).await?;
        }
    };
    Ok(())
}
