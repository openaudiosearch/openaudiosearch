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
struct FeedManager {
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
/// Opens a feed manager and start to track and watch all feeds in the database,
/// This will load all feeds from [CouchDB]
/// and then look for incoming feeds in the [ChangesStream].
/// It will periodically fetch the feeds and insert new items
/// as [Post]s and [Media]s into the database.
pub async fn run_manager(db: &CouchDB) -> anyhow::Result<()> {
    let mut manager = FeedManager::new();
    manager.init(db).await?;

    let tasks = watch_feeds(manager.store, db.clone())?;
    let watch_task = tokio::spawn({
        let db = db.clone();
        async move { watch_changes(db).await }
    });
    for handle in tasks.into_iter() {
        handle.await??;
    }
    watch_task.await??;
    Ok(())
}

fn watch_feeds(store: Store, db: CouchDB) -> Result<Vec<Task<()>>, RssError> {
    let mut tasks = Vec::new();

    for (id, feed) in store.clone().into_iter() {
        let settings = feed.value.settings;
        log::debug!(
            "Start to watch feed {} [{}] for updates",
            id,
            feed.value.url
        );
        let mut watcher = FeedWatcher::new(feed.value.url, settings)?;
        let db = db.clone();
        tasks.push(tokio::spawn(async move { watcher.watch(db).await }));
    }
    Ok(tasks)
}

async fn watch_changes(db: CouchDB) -> Result<(), RssError> {
    let last_seq = db.get_last_seq().await?;
    let mut stream = db.changes(Some(last_seq));
    let mut tasks = Vec::new();
    stream.set_infinite(true);
    let client = surf::client();
    while let Some(event) = stream.next().await {
        let event = event?;
        if let Some(doc) = event.doc {
            let _id = doc.id().to_string();
            let record = doc.into_typed_record::<types::Feed>();
            match record {
                Err(_err) => {}
                Ok(record) => {
                    let url = &record.value.url;
                    let settings = record.value.settings;
                    let mut watcher = FeedWatcher::with_client(client.clone(), url, settings)?;
                    let db = db.clone();
                    tasks.push(tokio::spawn(async move { watcher.watch(db).await }));
                }
            }
        }
    }
    Ok(())
}
