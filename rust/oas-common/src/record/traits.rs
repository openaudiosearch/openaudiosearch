use serde::de::DeserializeOwned;
use serde::Serialize;
use std::any::Any;
use std::fmt;

use super::error::ValidationError;

/// A trait to implement on value structs for typed [Record]s.
pub trait TypedValue:
    fmt::Debug + Any + Serialize + DeserializeOwned + std::clone::Clone + Send
{
    /// A string to uniquely identify this record type.
    const NAME: &'static str;

    /// Get a human-readable label for this record.
    ///
    /// This method is optional and returns None by default. Record types may implement this method
    /// to return a title or headline.
    fn label(&self) -> Option<&'_ str> {
        None
    }

    /// Get the guid string for this record type and an id string.
    fn guid(id: &str) -> String {
        format!("{}_{}", Self::NAME, id)
    }

    fn validate(&self) -> Result<(), ValidationError> {
        Ok(())
    }
}
