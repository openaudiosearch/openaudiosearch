use anyhow::Context;

pub mod couch;
// pub mod couch2;
pub mod index;
pub mod rss;
mod runtime;
pub mod server;
pub mod tasks;
pub mod util;

pub use oas_common as common;
pub use oas_common::{types, Record, Reference, TypedValue, UntypedRecord};
pub use runtime::Runtime;

/// Main application state.
///
/// This struct has instances to the mostly stateless clients to other services (CouchDB,
/// Elasticsearch). It should be cheap clone.
#[derive(Clone, Debug)]
pub struct State {
    pub db: couch::CouchDB,
    pub index_manager: index::IndexManager,
    pub tasks: tasks::CeleryManager,
}

impl State {
    /// Asynchronously init all services.
    ///
    /// Currently errors on the first failing init.
    pub async fn init_all(&mut self) -> anyhow::Result<()> {
        self.db
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
