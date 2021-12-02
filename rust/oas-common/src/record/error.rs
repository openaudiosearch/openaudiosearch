use serde::Serialize;
use std::fmt;
use thiserror::Error;

/// An error that occurs while encoding a record.
#[derive(Error, Debug)]
pub enum EncodingError {
    #[error("Serialization failed")]
    SerdeJson(#[from] serde_json::Error),
    #[error("Serialization did not return an object")]
    NotAnObject,
    #[error("Invalid patch")]
    Patch(#[from] json_patch::PatchError),
}

/// An error that occurs while decoding a record.
#[derive(Error, Debug)]
pub enum DecodingError {
    #[error("Deserialization failed")]
    SerdeJson(#[from] serde_json::Error),
    #[error("Type mismatch: expected {0}, got {1}")]
    TypeMismatch(String, String),
    #[error("Deserialization did not return an object")]
    NotAnObject,
}

#[derive(Debug, Serialize)]
pub struct ValidationError {
    message: String,
}

impl<E> From<E> for ValidationError
where
    E: std::error::Error + Send + 'static,
{
    fn from(e: E) -> Self {
        Self::from_error(e)
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl ValidationError {
    pub fn with_message(message: String) -> Self {
        Self { message }
    }
    pub fn from_error<E>(error: E) -> Self
    where
        E: std::error::Error + Send + 'static,
    {
        Self {
            message: format!("{}", error),
        }
    }
}
