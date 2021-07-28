pub mod mapping;
pub mod record;
pub mod reference;
pub mod resolver;
pub mod types;
pub mod util;

pub use record::*;
pub use reference::*;
pub use resolver::*;

pub trait ElasticMapping {
    fn elastic_mapping() -> Option<serde_json::Value>;
}
