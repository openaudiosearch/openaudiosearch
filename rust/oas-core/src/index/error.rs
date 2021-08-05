#[derive(thiserror::Error, Debug)]
pub enum IndexError {
    #[error("Elasticsearch error: {0}")]
    Elastic(#[from] elasticsearch::Error),
    #[error("Elasticsearch exception: {:?}", .0.error())]
    Exception(elasticsearch::http::response::Exception),
    #[error("Other: {0}")]
    Other(String),
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
}

impl From<elasticsearch::http::response::Exception> for IndexError {
    fn from(ex: elasticsearch::http::response::Exception) -> Self {
        Self::Exception(ex)
    }
}
