use super::error::RssError;
use super::FeedWatcher;
use crate::couch::CouchDB;
use oas_common::types;
use oas_common::TypedRecord;
use oas_common::TypedValue;
use std::collections::HashMap;
use tokio::task::JoinHandle;
use tokio_stream::StreamExt;

type Task<T> = JoinHandle<Result<T, RssError>>;
type Store = HashMap<String, TypedRecord<types::Feed>>;

use std::{
    thread,
    time::{Duration, Instant},
};

#[derive(Debug)]
pub struct FeedManager {
    store: HashMap<String, TypedRecord<types::Feed>>,
}

impl FeedManager {
    fn new() -> Self {
        let manager = Self {
            store: HashMap::new(),
        };
        return manager;
    }
    async fn init(&mut self, db: &CouchDB) -> anyhow::Result<()> {
        let records = db.get_all_records::<types::Feed>().await?;
        for record in records {
            eprintln!("RECORD: {:?}", record);
            self.store.insert(record.id().into(), record);
        }
        Ok(())
    }
}

pub async fn run_manager(db: &CouchDB) -> anyhow::Result<()> {
    let mut manager = FeedManager::new();
    manager.init(db).await?;
    //let store = Arc::new(Mutex::new(manager.store));
    let tasks = watch_feeds(manager.store)?;
    eprintln!("TASKS {:?}", tasks);
    eprintln!("START  CHANGES");
    watch_changes(db).await?;
    for handle in tasks.into_iter() {
        handle.await??;
    }
    Ok(())
}

fn watch_feeds(store: Store) -> Result<Vec<Task<()>>, RssError> {
    let mut tasks = Vec::new();

    for (id, feed) in store.clone().into_iter() {
        let mut watcher = FeedWatcher::new(feed.value.url)?;

        tasks.push(tokio::spawn(async move {
            eprintln!("Watch {}", id);
            watcher.watch().await
        }));
    }
    Ok(tasks)
}

/// Checks the CouchDB [ChangesStream] for incoming feed records.
async fn watch_changes(db: &CouchDB) -> Result<Vec<Task<()>>, RssError> {
    let mut stream = db.changes(None);
    let mut tasks = Vec::new();
    stream.set_infinite(true);
    while let Some(event) = stream.next().await {
        let event = event?;
        if let Some(doc) = event.doc {
            let _id = doc.id().to_string();
            let record = doc.into_typed_record::<types::Feed>();
            match record {
                Err(_err) => {}
                Ok(record) => match record.typ() {
                    types::Feed::NAME => {
                        let url = &record.value.url;
                        let mut watcher = FeedWatcher::new(url)?;
                        eprintln!("TASKS {:?}", tasks.len());
                        tasks.push(tokio::spawn(async move { watcher.watch().await }));
                    }
                    _ => {}
                },
            }
        }
    }
    eprintln!("AFTER ###############################");
    Ok(tasks)
}
