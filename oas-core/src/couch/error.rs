
use std::fmt;
use thiserror::Error;

use super::types::ErrorDetails;

#[derive(Error, Debug)]
pub enum CouchError {
    #[error("HTTP error: {0}")]
    Http(surf::Error),
    #[error("CouchDB error")]
    Couch(#[from] ErrorDetails),
    #[error("Serialization error")]
    Json(#[from] serde_json::Error),
    #[error("IO error")]
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
            write!(
                f,
                "CouchDB error for id {}: {} (reason: {})",
                id, self.error, self.reason
            )
        } else {
            write!(f, "CouchDB error: {} (reason: {})", self.error, self.reason)
        }
    }
}
