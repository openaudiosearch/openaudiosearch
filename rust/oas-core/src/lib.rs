use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use anyhow::Context;

pub mod couch;
// pub mod couch2;
pub mod index;
pub mod jobs;
pub mod rss;
mod runtime;
pub mod server;
pub mod store;
pub mod util;

use crate::rss::FeedManager;
use couch::{CouchDB, CouchManager};
pub use oas_common as common;
pub use oas_common::{types, Record, Reference, TypedValue, UntypedRecord};
pub use runtime::Runtime;
pub use store::RecordStore;

/// Main application state.
///
/// This struct has instances to the mostly stateless clients to other services (CouchDB,
/// Elasticsearch). It should be cheap clone.
#[derive(Clone, Debug)]
pub struct State {
    pub feed_manager: FeedManager,
    pub db_manager: CouchManager,
    pub db: couch::CouchDB,
    pub index_manager: index::IndexManager,
    pub jobs: jobs::JobManager,
    pub store: RecordStore,
    did_init: Arc<AtomicBool>,
}

impl State {
    pub fn new(
        db_manager: CouchManager,
        db: CouchDB,
        index_manager: index::IndexManager,
        feed_manager: FeedManager,
        jobs: jobs::JobManager,
    ) -> Self {
        let storage = db.clone();
        let store = RecordStore::with_storage(Box::new(storage));
        Self {
            db_manager,
            db,
            index_manager,
            feed_manager,
            jobs,
            store,
            did_init: Arc::new(AtomicBool::new(false)),
        }
    }
    /// Asynchronously init all services.
    ///
    /// Currently errors on the first failing init.
    pub async fn init_all(&mut self) -> anyhow::Result<()> {
        if self
            .did_init
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            return Ok(());
        }
        self.db_manager
            .init()
            .await
            .context("Failed to initialize CouchDB.")?;
        self.feed_manager
            .init(&self.db)
            .await
            .context("Failed to initialize RSS feed watcher")?;
        self.index_manager
            .init(Default::default())
            .await
            .context("Failed to initialize Elasticsearch.")?;
        Ok(())
    }
}
