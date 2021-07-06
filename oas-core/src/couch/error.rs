use std::fmt;
use thiserror::Error;

use super::types::ErrorDetails;

#[derive(Error, Debug)]
pub enum CouchError {
    #[error("HTTP: {0}")]
    Http(surf::Error),
    #[error("CouchDB: {0}")]
    Couch(#[from] ErrorDetails),
    #[error("Serialization: {0}")]
    Json(#[from] serde_json::Error),
    #[error("IO: {0}")]
    IO(#[from] std::io::Error),
    #[error("Error: {0}")]
    Other(String),
}

impl From<surf::Error> for CouchError {
    fn from(err: surf::Error) -> Self {
        Self::Http(err)
    }
}

impl std::error::Error for ErrorDetails {}

impl fmt::Display for ErrorDetails {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(id) = &self.id {
            write!(f, "{} (reason: {}, id {})", self.error, self.reason, id)
        } else {
            write!(f, "{} (reason: {})", self.error, self.reason)
        }
    }
}
