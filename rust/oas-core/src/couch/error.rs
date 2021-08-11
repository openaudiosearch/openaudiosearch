use oas_common::DecodingError;
use std::fmt;
use thiserror::Error;

use super::types::ErrorDetails;

#[derive(Error, Debug)]
pub enum CouchError {
    #[error("HTTP: {0}")]
    Http(reqwest::Error),
    #[error("CouchDB: {1}")]
    Couch(reqwest::StatusCode, ErrorDetails),
    #[error("Serialization: {0}")]
    Json(#[from] serde_json::Error),
    #[error("IO: {0}")]
    IO(#[from] std::io::Error),
    #[error("Other: {0}")]
    DecodingError(#[from] DecodingError),
    #[error("{0}")]
    Other(String),
}

impl CouchError {
    pub fn status_code(&self) -> Option<u16> {
        match self {
            Self::Http(err) => err.status().map(|s| s.as_u16()),
            Self::Couch(status, _) => Some((*status).into()),
            _ => None,
        }
    }
}

impl From<reqwest::Error> for CouchError {
    fn from(err: reqwest::Error) -> Self {
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
