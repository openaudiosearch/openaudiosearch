use clap::Clap;
use serde::{Deserialize, Serialize};
use std::time::Instant;

use super::crawlers::default_crawlers;
use super::*;
use crate::couch::{CouchDB, Doc, PutResult};

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

    /// Stop on first existing post.
    #[clap(long)]
    update: bool,

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
            update: false,
        }
    }

    pub fn crawl(url: Url, max_pages: Option<usize>, update: bool) -> Self {
        Self {
            url,
            crawl: true,
            max_pages,
            update,
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
    pub items: Vec<Record<Post>>,
    pub feed: FeedWatcher,
    pub put_result: Vec<PutResult>,
}

pub async fn crawler_loop(
    db: &CouchDB,
    opts: &CrawlOpts,
    crawler: &dyn Crawler, // crawler: T,
) -> RssResult<()> {
    let client = surf::Client::new();
    let mut url = opts.url.clone();
    let mut total = 0;
    let max_pages = opts.max_pages.unwrap_or(usize::MAX);
    let start = Instant::now();
    for _i in 0..max_pages {
        let feed_page = fetch_and_save_with_client(client.clone(), &db, &url).await?;

        // Check if the batch put to db contained any errors.
        // An error should occur when putting an existing ID
        // (i.e. existing URL).
        // TODO: Actually check the error.
        if !opts.update {
            let contains_existing = feed_page.put_result.iter().find_map(|result| match result {
                PutResult::Err(err) => Some(err),
                _ => None,
            });
            if let Some(err) = contains_existing {
                log::debug!("breaking crawl loop on {}", err);
                break;
            }
        }

        log::debug!(
            "imported {} items from {}",
            feed_page.items.len(),
            feed_page.url
        );

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

pub async fn fetch_and_save_with_client(
    client: surf::Client,
    db: &CouchDB,
    url: &Url,
) -> RssResult<FetchedFeedPage> {
    let mut feed = FeedWatcher::with_client(client, &url, None).unwrap();
    feed.load().await?;
    save_feed_to_db(db, feed).await
}

pub async fn fetch_and_save(db: &CouchDB, url: &Url) -> RssResult<FetchedFeedPage> {
    let mut feed = FeedWatcher::new(&url, None).unwrap();
    feed.load().await?;
    save_feed_to_db(db, feed).await
}

async fn save_feed_to_db(db: &CouchDB, feed: FeedWatcher) -> RssResult<FetchedFeedPage> {
    // let items = feed.into_medias()?;
    let mut posts = feed.into_posts()?;
    let mut docs = vec![];
    for post in posts.iter_mut() {
        let refs = post.extract_refs();
        let mut refs: Vec<Doc> = refs
            .into_iter()
            .map(|r| Doc::from_untyped_record(r))
            .collect();
        let post = Doc::from_typed_record(post.clone());
        docs.append(&mut refs);
        docs.push(post);
    }

    let put_result = db.put_bulk(docs).await?;
    // eprintln!("res {:#?}", put_result);

    let (success, error): (Vec<_>, Vec<_>) = put_result
        .iter()
        .partition(|r| matches!(r, PutResult::Ok(_)));

    log::info!(
        "saved {} docs from feed {} ({} success, {} error)",
        put_result.len(),
        feed.url,
        success.len(),
        error.len()
    );
    let feed_page = FetchedFeedPage {
        url: feed.url.clone(),
        feed,
        items: posts,
        put_result,
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
//     let records = feed.into_medias()?;
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
