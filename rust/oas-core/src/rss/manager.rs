use clap::Clap;
use oas_common::types;
use oas_common::TypedRecord;
use std::collections::HashMap;
use tokio::task::JoinHandle;
use tokio_stream::StreamExt;

use super::error::RssError;
use super::mapping::{AllMappings, MappingManager};
use super::FeedWatcher;
use crate::couch::CouchDB;

type Task<T> = JoinHandle<Result<T, RssError>>;
// type Store = HashMap<String, TypedRecord<types::Feed>>;

#[derive(Debug, Clone, Default, Clap)]
pub struct FeedManagerOpts {
    #[clap(long)]
    pub mapping_file: Option<String>,
}

impl FeedManagerOpts {
    pub fn with_mapping_file(f: String) -> Self {
        Self {
            mapping_file: Some(f),
        }
    }
}

#[derive(Debug)]
struct FeedManager {
    store: HashMap<String, TypedRecord<types::Feed>>,
    mapping_manager: MappingManager,
    opts: FeedManagerOpts,
}

impl FeedManager {
    fn new(opts: FeedManagerOpts) -> Self {
        let mapping_manager = if let Some(file) = &opts.mapping_file {
            MappingManager::with_file(&file)
        } else {
            MappingManager::new()
        };
        Self {
            store: HashMap::new(),
            opts,
            mapping_manager,
        }
    }
    async fn init(&mut self, db: &CouchDB) -> anyhow::Result<()> {
        let records = db.get_all_records::<types::Feed>().await?;
        for record in records {
            self.store.insert(record.id().into(), record);
        }
        self.mapping_manager.init().await?;
        Ok(())
    }
}
/// Opens a feed manager and start to track and watch all feeds in the database,
/// This will load all feeds from [CouchDB]
/// and then look for incoming feeds in the [ChangesStream].
/// It will periodically fetch the feeds and insert new items
/// as [Post]s and [Media]s into the database.
pub async fn run_manager(db: &CouchDB, opts: FeedManagerOpts) -> anyhow::Result<()> {
    let mut manager = FeedManager::new(opts);
    manager.init(db).await?;
    // TODO: Remove clone of mapping.
    let mapping = manager.mapping_manager.to_field_hashmap();
    let tasks = watch_feeds(manager, db.clone())?;
    let watch_task = tokio::spawn({
        let db = db.clone();
        async move { watch_changes(mapping, db).await }
    });
    for handle in tasks.into_iter() {
        handle.await??;
    }
    watch_task.await??;
    Ok(())
}

fn watch_feeds(manager: FeedManager, db: CouchDB) -> Result<Vec<Task<()>>, RssError> {
    let mut tasks = Vec::new();
    let store = manager.store;
    let mapping = manager.mapping_manager.to_field_hashmap();
    for (id, feed) in store.into_iter() {
        let settings = feed.value.settings;
        log::debug!(
            "Start to watch feed {} [{}] for updates",
            id,
            feed.value.url
        );
        let mut watcher = FeedWatcher::new(feed.value.url, settings, mapping.clone())?;
        let db = db.clone();
        tasks.push(tokio::spawn(async move { watcher.watch(db).await }));
    }
    Ok(tasks)
}

async fn watch_changes(mapping: AllMappings, db: CouchDB) -> Result<(), RssError> {
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
                    let mut watcher =
                        FeedWatcher::with_client(client.clone(), url, settings, mapping.clone())?;
                    let db = db.clone();
                    tasks.push(tokio::spawn(async move { watcher.watch(db).await }));
                }
            }
        }
    }
    Ok(())
}
