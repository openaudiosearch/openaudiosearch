use clap::Clap;
use oas_common::types;
use oas_common::types::Feed;
use oas_common::util::id_from_hashed_string;
use oas_common::TypedRecord;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
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

#[derive(Debug, Clone)]
pub struct FeedManager {
    pub(crate) inner: Arc<Mutex<FeedManagerInner>>,
}

impl FeedManager {
    pub fn new(opts: FeedManagerOpts) -> Self {
        Self {
            inner: Arc::new(Mutex::new(FeedManagerInner::new(opts))),
        }
    }

    /// Init initial feed state.
    pub async fn init(&self, db: &CouchDB) -> anyhow::Result<()> {
        self.inner.lock().await.init(db).await
    }

    /// Refetch a single feed by ID or URL.
    pub async fn refetch(&self, db: &CouchDB, id_or_url: &str) -> anyhow::Result<()> {
        self.inner.lock().await.refetch(db, id_or_url).await
    }

    /// Opens a feed manager and start to track and watch all feeds in the database,
    /// This will load all feeds from [CouchDB]
    /// and then look for incoming feeds in the [ChangesStream].
    /// It will periodically fetch the feeds and insert new items
    /// as [Post]s and [Media]s into the database.
    pub async fn run_watch(self, db: CouchDB) -> anyhow::Result<()> {
        run_watch(self, db).await
    }

    // pub async fn run_crawl(self, db: CouchDB) -> anyhow::Result<()> {
    // run_crawl(self, db).await
    // }
}

#[derive(Debug)]
pub struct FeedManagerInner {
    store: HashMap<String, TypedRecord<types::Feed>>,
    mapping_manager: MappingManager,
    opts: FeedManagerOpts,
    init: bool
}

impl FeedManagerInner {
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
            init: false
        }
    }

    async fn init(&mut self, db: &CouchDB) -> anyhow::Result<()> {
        if self.init {
            return Ok(())
        }
        self.mapping_manager.init().await?;
        let records = db.get_all_records::<types::Feed>().await?;
        for record in records {
            self.store.insert(record.id().into(), record);
        }
        self.init = true;
        Ok(())
    }

    pub async fn refetch(&mut self, db: &CouchDB, id_or_url: &str) -> anyhow::Result<()> {
        self.init(&db).await?;
        let table = db.table::<Feed>();
        let feed = match table.get(&id_or_url).await {
            Ok(feed) => feed,
            Err(_) => match table.get(&id_from_hashed_string(&id_or_url)).await {
                Ok(feed) => feed,
                Err(err) => return Err(err.into()),
            },
        };

        log::info!("Refetch feed {} ({})", feed.id(), feed.value.url);

        let mut watcher = FeedWatcher::new(
            &feed.value.url.clone(),
            feed.value.settings.clone(),
            self.mapping_manager.to_field_hashmap(),
            Some(feed),
        )?;

        watcher.load().await?;
        watcher.save(&db, true).await?;
        Ok(())
    }
}

async fn run_watch(manager: FeedManager, db: CouchDB) -> anyhow::Result<()> {
    let tasks = start_feed_tasks(&manager, db.clone()).await?;
    let mapping = {
        manager
            .inner
            .lock()
            .await
            .mapping_manager
            .to_field_hashmap()
    };
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

async fn start_feed_tasks(manager: &FeedManager, db: CouchDB) -> Result<Vec<Task<()>>, RssError> {
    let mut tasks = Vec::new();
    let manager = manager.inner.lock().await;
    let store = &manager.store;
    let mapping = manager.mapping_manager.to_field_hashmap();
    for (id, feed) in store.iter() {
        let settings = feed.value.settings.clone();
        log::debug!(
            "Start to watch feed {} [{}] for updates",
            id,
            feed.value.url
        );
        let mut watcher = FeedWatcher::new(
            &feed.value.url,
            settings,
            mapping.clone(),
            Some(feed.clone()),
        )?;
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
    let client = reqwest::Client::new();
    while let Some(event) = stream.next().await {
        let event = event?;
        if let Some(doc) = event.doc {
            let _id = doc.id().to_string();
            let record = doc.into_typed_record::<types::Feed>();
            match record {
                Err(_err) => {}
                Ok(record) => {
                    let url = record.value.url.clone();
                    let settings = record.value.settings.clone();
                    let mut watcher = FeedWatcher::with_client(
                        client.clone(),
                        &url,
                        settings,
                        mapping.clone(),
                        Some(record),
                    )?;
                    let db = db.clone();
                    tasks.push(tokio::spawn(async move { watcher.watch(db).await }));
                }
            }
        }
    }
    Ok(())
}
