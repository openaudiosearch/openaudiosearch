use clap::Clap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::{sync::Arc, time::Instant};

use crate::couch::{CouchDB, Doc, PutResult};

use super::*;
pub enum Next {
    Finished,
    NextPage(Url),
}

#[derive(Clap, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CrawlOpts {
    /// Feed URL to ingest
    url: Url,
    /// Crawl a paginated feed recursively.
    #[clap(short, long)]
    crawl: bool,

    /// Max number of pages to crawl.
    #[clap(long)]
    max_pages: Option<usize>,
}

impl CrawlOpts {
    pub fn new(url: Url) -> Self {
        Self {
            url,
            crawl: true,
            max_pages: None,
        }
    }

    pub fn crawl(url: Url, max_pages: Option<usize>) -> Self {
        Self {
            url,
            crawl: true,
            max_pages,
        }
    }
}

// struct FeedFetchResult {
//     url: Url,
//     feed: Feed,
//     records: Vec<Record>,
//     db_result: Vec<PutResult>
// }

pub async fn fetch_and_save(db: &CouchDB, url: &Url) -> RssResult<(Feed, Vec<PutResult>)> {
    let mut feed = Feed::new(url)?;
    feed.load().await?;
    let records = feed.into_audio_objects()?;
    // let guids: Vec<Option<String>> = records
    //     .iter()
    //     .map(|item| item.value.identifier.clone())
    //     .collect();
    // let guids: Vec<String> = guids.into_iter().map(|i| i.unwrap()).collect();
    let res = db.put_bulk_update(records.into()).await?;
    Ok((feed, res))
}

pub fn default_crawlers() -> Vec<Pin<Box<dyn Crawler>>> {
    vec![Box::pin(frn::FrnCrawler {}), Box::pin(cba::CbaCrawler {})]
}

pub async fn crawl_and_save(db: &CouchDB, opts: &CrawlOpts) -> RssResult<()> {
    let url = &opts.url;
    let crawlers = default_crawlers();

    let domain = url
        .domain()
        .ok_or_else(|| RssError::MissingCrawlRule(url.to_string()))?;
    for crawler in crawlers.into_iter() {
        if crawler.domains().contains(&domain.to_string()) {
            return crawler_loop(db, opts, &*crawler).await;
        }
    }
    Err(RssError::MissingCrawlRule(domain.to_string()))
}

#[derive(Debug, Clone)]
pub struct Request {
    pub url: Url,
    pub items: Arc<Vec<Record<AudioObject>>>,
}

pub async fn crawler_loop(
    db: &CouchDB,
    opts: &CrawlOpts,
    crawler: &dyn Crawler, // crawler: T,
) -> RssResult<()> {
    let mut url = opts.url.clone();
    let mut total = 0;
    let max_pages = opts.max_pages.unwrap_or(usize::MAX);
    let start = Instant::now();
    for _i in 0..max_pages {
        let request = feed_loop_next(&db, &url).await?;
        total += request.items.len();
        let next = crawler.next(request).await?;
        url = match next {
            Next::Finished => break,
            Next::NextPage(url) => url,
        };
    }
    Ok(())
}

pub async fn feed_loop<F, Fut>(db: &CouchDB, opts: &CrawlOpts, callback: F) -> RssResult<()>
where
    F: Send + 'static + Fn(Request) -> Fut,
    Fut: Send + 'static + Future<Output = anyhow::Result<Next>>,
{
    let mut url = opts.url.clone();
    let mut total = 0;
    let max_pages = opts.max_pages.unwrap_or(usize::MAX);
    let start = Instant::now();
    for _i in 0..max_pages {
        let request = feed_loop_next(&db, &url).await?;
        total += request.items.len();
        let next = callback(request).await?;
        url = match next {
            Next::Finished => break,
            Next::NextPage(url) => url,
        };
    }
    // let duration = start.elapsed();
    // let per_second = total as f32 / duration.as_secs_f32();
    // eprintln!(
    //     "Imported {} items in {:?} ({}/s)",
    //     total, duration, per_second
    // );
    Ok(())
}

pub async fn feed_loop_next(db: &CouchDB, url: &Url) -> RssResult<Request> {
    let mut feed = Feed::new(&url).unwrap();
    feed.load().await?;
    let items = feed.into_audio_objects()?;
    let docs: Vec<Doc> = items.iter().map(|r| r.clone().into()).collect();
    let _put_result = db.put_bulk(docs).await?;
    let items = Arc::new(items);
    let request = Request {
        url: url.clone(),
        items,
    };
    Ok(request)
}

fn query_map(url: &Url) -> HashMap<String, String> {
    url.query_pairs().into_owned().collect()
}

fn set_query_map(url: &mut Url, map: &HashMap<String, String>) {
    url.query_pairs_mut().clear().extend_pairs(map.iter());
}

fn insert_or_add(map: &mut HashMap<String, String>, key: &str, default: usize, add: usize) {
    if let Some(value) = map.get_mut(key) {
        let num: Result<usize, _> = value.parse();
        let num = num.map_or(default, |num| num + add);
        *value = num.to_string();
    } else {
        map.insert(key.into(), default.to_string());
    }
}

#[async_trait::async_trait]
pub trait Crawler: Send + Sync {
    async fn next(&self, _request: Request) -> anyhow::Result<Next> {
        Ok(Next::Finished)
    }
    fn domains(&self) -> Vec<String> {
        vec![]
    }
}

mod frn {
    pub struct FrnCrawler {}
    use super::*;

    #[async_trait::async_trait]
    impl Crawler for FrnCrawler {
        async fn next(&self, request: Request) -> anyhow::Result<Next> {
            match request.items.len() {
                0 => Ok(Next::Finished),
                _ => {
                    let len = request.items.len();
                    let mut url = request.url;
                    let mut params = query_map(&url);
                    insert_or_add(&mut params, "start", len, len);
                    set_query_map(&mut url, &params);
                    Ok(Next::NextPage(url))
                }
            }
        }

        fn domains(&self) -> Vec<String> {
            vec![
                "freie-radios.net".to_string(),
                "www.freie-radios.net".to_string(),
            ]
        }
    }
    // pub async fn crawl_callback(request: Request) -> anyhow::Result<Next> {
    //     let items = request.items;
    //     let mut url = request.url;
    //     match items.len() {
    //         0 => Ok(Next::Finished),
    //         _ => {
    //             let mut params = query_map(&url);
    //             insert_or_add(&mut params, "start", items.len(), items.len());
    //             set_query_map(&mut url, &params);
    //             Ok(Next::NextPage(url))
    //         }
    //     }
    // }
}

mod cba {
    use super::*;
    pub struct CbaCrawler {}
    #[async_trait::async_trait]
    impl Crawler for CbaCrawler {
        async fn next(&self, request: Request) -> anyhow::Result<Next> {
            let len = request.items.len();
            match len {
                0 => Ok(Next::Finished),
                _ => {
                    let mut url = request.url;
                    let mut params = query_map(&url);
                    insert_or_add(&mut params, "offset", len, len);
                    set_query_map(&mut url, &params);
                    Ok(Next::NextPage(url))
                }
            }
        }

        fn domains(&self) -> Vec<String> {
            vec!["cba.media".to_string(), "cba.fro.at".to_string()]
        }
    }
}
