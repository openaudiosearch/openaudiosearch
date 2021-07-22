pub mod couch;
pub mod index;
pub mod rss;
pub mod server;
pub mod tasks;
pub mod util;

pub use oas_common::*;

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
    pub async fn init_all(&self) -> anyhow::Result<()> {
        self.db.init().await?;
        self.index_manager.init(Default::default()).await?;
        Ok(())
    }
}
