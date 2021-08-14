use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use anyhow::Context;

pub mod couch;
// pub mod couch2;
pub mod index;
pub mod rss;
mod runtime;
pub mod server;
pub mod tasks;
pub mod util;

use couch::{CouchDB, CouchManager};
pub use oas_common as common;
pub use oas_common::{types, Record, Reference, TypedValue, UntypedRecord};
pub use runtime::Runtime;

/// Main application state.
///
/// This struct has instances to the mostly stateless clients to other services (CouchDB,
/// Elasticsearch). It should be cheap clone.
#[derive(Clone, Debug)]
pub struct State {
    pub db_manager: CouchManager,
    pub db: couch::CouchDB,
    pub index_manager: index::IndexManager,
    pub tasks: tasks::CeleryManager,
    did_init: Arc<AtomicBool>,
}

impl State {
    pub fn new(
        db_manager: CouchManager,
        db: CouchDB,
        index_manager: index::IndexManager,
        tasks: tasks::CeleryManager,
    ) -> Self {
        Self {
            db_manager,
            db,
            index_manager,
            tasks,
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
        self.index_manager
            .init(Default::default())
            .await
            .context("Failed to initialize Elasticsearch.")?;
        self.tasks
            .init()
            .await
            .context("Failed to initialize Celery/Redis task manager.")?;
        Ok(())
    }
}
