use crate::State;
use oas_common::types;

use super::{Feed, RssError};

pub struct FeedManager {
    feeds: Vec<Feed>,
}

impl FeedManager {
    pub fn new() -> Self {
        Self { feeds: vec![] }
    }

    pub async fn run(db: &CouchDB) {
        let db = state.db;
        let feed_records = db.get_all_records::<types::Feed>().await?;
        let mut errors = vec![];
        let mut feeds = vec![];
        for feed_record in feed_records {
            let url = feed_record.value.url;
            let feed = Feed::new(url);
            match feed {
                Ok(feed) => feeds.push(feed),
                Err(e) => errors.push((err, url)),
            }
        }
    }
}

pub async fn feed_task_loop(db: &CouchDB, feed: Feed) -> Result<(), RssError> {
    loop {
        feed.load().await?;
        let items = feed.into_medias()?;
        let docs: Vec<Doc> = items.iter().map(|r| r.clone().into()).collect();
        let res = db.put_bulk(docs).await?;
    }
}
