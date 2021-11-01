use clap::Parser;
use serde::{Deserialize, Serialize};
use std::time::Instant;
use url::Url;

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

#[derive(Parser)]
pub struct FetchOpts {
    /// Feed URL
    url: Url,
    /// Force update
    #[clap(short, long)]
    update: bool,
}

#[derive(Parser, Serialize, Deserialize)]
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
    pub items: Vec<UntypedRecord>,
    pub feed: FeedWatcher,
    pub put_result: Vec<PutResult>,
}

pub async fn crawler_loop(
    db: &CouchDB,
    opts: &CrawlOpts,
    crawler: &dyn Crawler, // crawler: T,
) -> RssResult<()> {
    let client = reqwest::Client::new();
    let mut url = opts.url.clone();
    let mut total = 0;
    let max_pages = opts.max_pages.unwrap_or(usize::MAX);
    let start = Instant::now();
    for _i in 0..max_pages {
        log::debug!("fetching {}", url);
        let feed_page = fetch_and_save_with_client(client.clone(), &db, &url, opts.update).await?;

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
            Next::Finished => {
                log::debug!("breaking crawl loop: finished");
                break;
            }
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
    client: reqwest::Client,
    db: &CouchDB,
    url: &Url,
    update: bool,
) -> RssResult<FetchedFeedPage> {
    let mut feed = FeedWatcher::with_client(client, &url, None, Default::default(), None).unwrap();
    feed.load().await?;
    let (put_result, records) = feed.save(&db, update).await?;
    let feed_page = FetchedFeedPage {
        url: feed.url.clone(),
        feed,
        items: records,
        put_result,
    };
    Ok(feed_page)
}

pub async fn fetch_and_save(db: &CouchDB, opts: &FetchOpts) -> RssResult<FetchedFeedPage> {
    let mut feed = FeedWatcher::new(&opts.url, None, Default::default(), None).unwrap();
    feed.load().await?;
    let (put_result, records) = feed.save(&db, opts.update).await?;
    let feed_page = FetchedFeedPage {
        url: feed.url.clone(),
        feed,
        items: records,
        put_result,
    };
    Ok(feed_page)
}
