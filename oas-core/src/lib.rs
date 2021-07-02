pub mod celery;
pub mod couch;
pub mod elastic;
pub mod rss;
pub mod server;
pub mod tasks;
pub mod util;

pub use oas_common::*;

pub struct State {
    pub db: couch::CouchDB,
    pub elastic_index: String,
}

impl State {
    // pub fn init() -> Self {}
}
