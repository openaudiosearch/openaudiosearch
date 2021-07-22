mod elastic;
mod error;
mod manager;

pub use elastic::{Config, Index};
pub use error::IndexError;
pub use manager::{IndexManager, InitOpts};
