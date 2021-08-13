mod guid;
pub mod mapping;
pub mod record;
mod record_map;
pub mod reference;
pub mod resolver;
pub mod task;
pub mod types;
pub mod util;

pub use guid::Guid;
pub use record::*;
pub use record_map::*;
pub use reference::*;
pub use resolver::*;

pub trait ElasticMapping {
    fn elastic_mapping() -> Option<serde_json::Value>;
}
