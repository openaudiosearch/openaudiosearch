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
            self.store.insert(record.id().into(), record);
        }
        Ok(())
    }
}
/// Starts the [Feed Manager],
/// which loads the current feeds from [CouchDB]
/// and then looks for incoming feeds in the [Change Stream].
pub async fn run_manager(db: &CouchDB) -> anyhow::Result<()> {
    let mut manager = FeedManager::new();
    manager.init(db).await?;

    let tasks = watch_feeds(manager.store, db.clone())?;
    let watch_task = tokio::spawn({
        let db = db.clone();
        async move { watch_changes(db).await }
    });
    watch_task.await??;

    for handle in tasks.into_iter() {
        handle.await??;
    }
    Ok(())
}

fn watch_feeds(store: Store, db: CouchDB) -> Result<Vec<Task<()>>, RssError> {
    let mut tasks = Vec::new();

    for (id, feed) in store.clone().into_iter() {
        let mut watcher = FeedWatcher::new(feed.value.url)?;
        let db = db.clone();
        tasks.push(tokio::spawn(async move {
            eprintln!("Watch {}", id);
            watcher.watch(db).await
        }));
    }
    Ok(tasks)
}

async fn watch_changes(db: CouchDB) -> Result<(), RssError> {
    let last_seq = db.get_last_seq().await?;
    let mut stream = db.changes(Some(last_seq));
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
                        //eprintln!("TASKS {:?}", tasks.len());
                        let db = db.clone();
                        tasks.push(tokio::spawn(async move { watcher.watch(db).await }));
                    }
                    _ => {}
                },
            }
        }
    }
    Ok(())
}
