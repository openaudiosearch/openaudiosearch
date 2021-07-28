use thiserror::Error;
use url::ParseError;

pub type RssResult<T> = Result<T, RssError>;

#[derive(Error, Debug)]
pub enum RssError {
    #[error("HTTP error: {0}")]
    Http(surf::Error),
    #[error("Serialization error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Remote error: {}", .0.status())]
    RemoteHttpError(Box<surf::Response>),
    // #[error("IO error")]
    // IO(#[from] std::io::Error),
    #[error("RSS error")]
    RSS(#[from] rss::Error),
    #[error("Feed must be loaded first or was invalid")]
    NoChannel,
    #[error("No crawl rule defined for domain: {0}")]
    MissingCrawlRule(String),
    #[error("Invalid URL: {0}")]
    InvalidUrl(#[from] ParseError),
    #[error("Error: {0}")]
    Other(String),
    #[error("CouchError: {0}")]
    Couch(#[from] crate::couch::CouchError),
    #[error("Error: {0}")]
    Any(#[from] anyhow::Error),
}

impl From<surf::Error> for RssError {
    fn from(err: surf::Error) -> Self {
        Self::Http(err)
    }
}
