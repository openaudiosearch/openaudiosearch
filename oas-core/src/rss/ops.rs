use clap::Clap;
use serde::{Deserialize, Serialize};



use std::{time::Instant};

use crate::couch::{CouchDB, Doc};

use super::crawlers::default_crawlers;

use super::*;

pub enum Next {
    Finished,
    NextPage(Url),
}

#[async_trait::async_trait]
pub trait Crawler: Send + Sync {
    async fn next(&self, _feed_page: FetchedFeedPage) -> anyhow::Result<Next> {
        Ok(Next::Finished)
    }
    fn domains(&self) -> Vec<String> {
        vec![]
    }
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
pub struct FetchedFeedPage {
    pub url: Url,
    pub items: Vec<Record<AudioObject>>,
    pub feed: Feed,
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
        let feed_page = fetch_and_save(&db, &url).await?;
        total += feed_page.items.len();
        let next = crawler.next(feed_page).await?;
        url = match next {
            Next::Finished => break,
            Next::NextPage(url) => url,
        };
    }
    let duration = start.elapsed();
    let per_second = total as f32 / duration.as_secs_f32();
    log::info!(
        "Imported {} items in {:?} ({}/s) from {}",
        total,
        duration,
        per_second,
        url
    );
    Ok(())
}

pub async fn fetch_and_save(db: &CouchDB, url: &Url) -> RssResult<FetchedFeedPage> {
    let mut feed = Feed::new(&url).unwrap();
    feed.load().await?;
    let items = feed.into_audio_objects()?;
    let docs: Vec<Doc> = items.iter().map(|r| r.clone().into()).collect();
    let _put_result = db.put_bulk(docs).await?;
    let feed_page = FetchedFeedPage {
        url: url.clone(),
        feed,
        items,
    };
    Ok(feed_page)
}

// struct FeedFetchResult {
//     url: Url,
//     feed: Feed,
//     records: Vec<Record>,
//     db_result: Vec<PutResult>
// }
// pub async fn fetch_and_save(db: &CouchDB, url: &Url) -> RssResult<(Feed, Vec<PutResult>)> {
//     let mut feed = Feed::new(url)?;
//     feed.load().await?;
//     let records = feed.into_audio_objects()?;
//     let res = db.put_bulk_update(records.into()).await?;
//     Ok((feed, res))
// }
// pub async fn feed_loop<F, Fut>(db: &CouchDB, opts: &CrawlOpts, callback: F) -> RssResult<()>
// where
//     F: Send + 'static + Fn(FetchedFeedPage) -> Fut,
//     Fut: Send + 'static + Future<Output = anyhow::Result<Next>>,
// {
//     let mut url = opts.url.clone();
//     let mut total = 0;
//     let max_pages = opts.max_pages.unwrap_or(usize::MAX);
//     let start = Instant::now();
//     for _i in 0..max_pages {
//         let feed_page = feed_loop_next(&db, &url).await?;
//         total += feed_page.items.len();
//         let next = callback(feed_page).await?;
//         url = match next {
//             Next::Finished => break,
//             Next::NextPage(url) => url,
//         };
//     }
//     let duration = start.elapsed();
//     let per_second = total as f32 / duration.as_secs_f32();
//     log::info!(
//         "Imported {} items in {:?} ({}/s) from {}",
//         total,
//         duration,
//         per_second,
//         url
//     );
//     Ok(())
// }
// pub async fn crawl_callback(feed_page: FetchedFeedPage) -> anyhow::Result<Next> {
//     let items = feed_page.items;
//     let mut url = feed_page.url;
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
