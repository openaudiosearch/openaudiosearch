use anyhow::anyhow;
use async_std::stream::StreamExt;
use clap::Clap;
use oas_common::types::AudioObject;
use oas_common::{mapping, Record, TypedValue};
use oas_core::couch::{Doc, DocMeta};
use oas_core::rss;
use oas_core::rss::Feed;
use oas_core::server::{run_server, ServerOpts};
use oas_core::util::*;
use oas_core::State;
use oas_core::{couch, elastic, tasks};
use std::future::Future;
use std::{collections::HashMap, time};
use url::Url;

const DEFAULT_HOST: &str = "http://localhost:5984";
const DEFAULT_DB: &str = "oas_test2";

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
    /// Search for records
    Search(SearchOpts),
    /// Fetch a RSS feed
    Feed(FeedCommands),
    /// Create a test task
    Task,
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

#[async_std::main]
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

    let elastic_index = "rust-test-1".to_string();

    // TODO: Where does DB init happen?
    // db.init().await?;

    let state = State { db, elastic_index };

    let result = match args.command {
        Command::Watch(opts) => run_watch(state, opts).await,
        Command::List => run_list(state).await,
        Command::Debug => run_debug(state).await,
        Command::Index => run_index(state).await,
        Command::Search(opts) => run_search(state, opts).await,
        Command::Feed(opts) => run_feed(state, opts.command).await,
        Command::Task => run_task().await,
        Command::Server(opts) => run_server(opts).await,
    };
    result
}

async fn run_task() -> anyhow::Result<()> {
    tasks::run_celery().await?;
    // faktory::run_faktory().await?;
    Ok(())
}

async fn run_list(state: State) -> anyhow::Result<()> {
    let _db = state.db;
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
    //     docs.clone().into_typed_docs::<Record<AudioObject>>()
    // );
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
    let mut stream = db.changes(opts.since);
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
    let index_name = state.elastic_index;
    let client = elastic::create_client()?;
    // let index_name = "rust-test-1".to_string();
    // let client = elastic::create_client()?;
    elastic::ensure_index(&client, &index_name, true).await?;

    let start = time::Instant::now();

    let batch_size = 1000;
    let mut batch = vec![];
    let mut count = 0;
    let mut stream = db.changes(None);
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

async fn run_search(state: State, opts: SearchOpts) -> anyhow::Result<()> {
    let client = elastic::create_client()?;
    let records = elastic::find_records(&client, &state.elastic_index, &opts.query).await?;
    let records: Vec<Record<AudioObject>> = records
        .into_iter()
        .filter_map(|r| r.into_typed_record::<AudioObject>().ok())
        .collect();
    debug_print_records(&records[..]);
    Ok(())
}

async fn run_feed(state: State, command: FeedCommand) -> anyhow::Result<()> {
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

// mod feed {
//     use std::{sync::Arc, time::Instant};

//     use super::*;
//     pub enum Next {
//         Finished,
//         NextPage(Url),
//     }

//     pub async fn fetch_single(state: State, opts: FeedOpts) -> anyhow::Result<()> {
//         let url = opts.url;
//         let mut feed = Feed::new(&url)?;
//         feed.load().await?;
//         let records = feed.into_audio_objects()?;

//         println!("len: {}", records.len());
//         let guids: Vec<Option<String>> = records
//             .iter()
//             .map(|item| item.value.identifier.clone())
//             .collect();
//         let guids: Vec<String> = guids.into_iter().map(|i| i.unwrap()).collect();
//         println!("guids: {:#?}", guids);
//         let res = state
//             .db
//             .put_bulk_update(records.into())
//             .await
//             .map_err(|e| anyhow!(e))?;
//         eprintln!("couch res: {:?}", res);
//         Ok(())
//     }

//     pub async fn fetch_all(state: State, opts: FeedOpts) -> anyhow::Result<()> {
//         // this is the callack for freie-radios.net feeds
//         // let callback = |url: &Url, feed: &Feed, items: &[Record<AudioObject>]| async move {
//         // };

//         let url = &opts.url;
//         let domain = url.domain().ok_or(anyhow!("Invalid URL: {}", url))?;
//         match domain {
//             "freie-radios.net" | "www.freie-radios.net" => {
//                 feed_loop(state, opts, frn::crawl_callback).await
//             }
//             "cba.fro.at" => feed_loop(state, opts, cba::crawl_callback).await,
//             domain => Err(anyhow!("No crawl rule defined for domain {}", domain)),
//         }
//         // feed_loop(state, opts, closure).await?;
//         // Ok(())
//     }

//     #[derive(Debug, Clone)]
//     pub struct Request {
//         pub url: Url,
//         pub items: Arc<Vec<Record<AudioObject>>>,
//     }

//     pub async fn feed_loop<F, Fut>(state: State, opts: FeedOpts, callback: F) -> anyhow::Result<()>
//     where
//         F: Send + 'static + Fn(Request) -> Fut,
//         Fut: Send + 'static + Future<Output = anyhow::Result<Next>>,
//     {
//         let mut url = opts.url;
//         let mut total = 0;
//         let max = opts.crawl_max.unwrap_or(usize::MAX);
//         let start = Instant::now();
//         for _i in 0..max {
//             eprintln!("fetch {}", url);
//             let (req, next) = feed_loop_next(&state, &url, &callback).await?;
//             total += req.items.len();
//             url = match next {
//                 Next::Finished => break,
//                 Next::NextPage(url) => url,
//             };
//         }
//         let duration = start.elapsed();
//         let per_second = total as f32 / duration.as_secs_f32();
//         eprintln!(
//             "Imported {} items in {:?} ({}/s)",
//             total, duration, per_second
//         );
//         Ok(())
//     }

//     pub async fn feed_loop_next<F, Fut>(
//         state: &State,
//         url: &Url,
//         callback: &F,
//     ) -> anyhow::Result<(Request, Next)>
//     where
//         F: Send + 'static + Fn(Request) -> Fut,
//         Fut: Send + 'static + Future<Output = anyhow::Result<Next>>,
//     {
//         let mut feed = Feed::new(&url).unwrap();
//         feed.load().await?;
//         let items = feed.into_audio_objects()?;

//         debug_print_records(&items);
//         let docs: Vec<Doc> = items.iter().map(|r| r.clone().into()).collect();
//         let _put_result = state.db.put_bulk(docs).await?;
//         // eprintln!(

//         let items = Arc::new(items);
//         let request = Request {
//             url: url.clone(),
//             items,
//         };
//         let next = callback(request.clone()).await?;
//         Ok((request, next))
//     }

//     fn query_map(url: &Url) -> HashMap<String, String> {
//         url.query_pairs().into_owned().collect()
//     }

//     fn set_query_map(url: &mut Url, map: &HashMap<String, String>) {
//         url.query_pairs_mut().clear().extend_pairs(map.iter());
//     }

//     fn insert_or_add(map: &mut HashMap<String, String>, key: &str, default: usize, add: usize) {
//         if let Some(value) = map.get_mut(key) {
//             let num: Result<usize, _> = value.parse();
//             let num = num.map_or(default, |num| num + add);
//             *value = num.to_string();
//         } else {
//             map.insert(key.into(), default.to_string());
//         }
//     }

//     mod frn {
//         use super::*;
//         pub async fn crawl_callback(request: Request) -> anyhow::Result<Next> {
//             let items = request.items;
//             let mut url = request.url;
//             match items.len() {
//                 0 => Ok(Next::Finished),
//                 _ => {
//                     let mut params = query_map(&url);
//                     insert_or_add(&mut params, "start", items.len(), items.len());
//                     set_query_map(&mut url, &params);
//                     Ok(Next::NextPage(url))
//                 }
//             }
//         }
//     }

//     mod cba {
//         use super::*;
//         pub async fn crawl_callback(request: Request) -> anyhow::Result<Next> {
//             let items = request.items;
//             let mut url = request.url;
//             match items.len() {
//                 0 => Ok(Next::Finished),
//                 _ => {
//                     let mut params = query_map(&url);
//                     insert_or_add(&mut params, "offset", items.len(), items.len());
//                     set_query_map(&mut url, &params);
//                     Ok(Next::NextPage(url))
//                 }
//             }
//         }
//     }
// }
