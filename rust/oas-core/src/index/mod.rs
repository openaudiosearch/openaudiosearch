mod error;
mod index;
mod manager;

pub use error::IndexError;
pub use index::{Config, Index};
pub use manager::{IndexManager, InitOpts};
