use crate::{Record, TypedValue, UntypedRecord};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub use crate::resolver::{ResolveError, Resolver};

/// Helper funtction to extract all loaded refences from a slice of references into
/// [UntypedRecord]s, mutating the references into IDs.
pub fn extract_refs<T: TypedValue>(refs: &mut [Reference<T>]) -> Vec<UntypedRecord> {
    refs.iter_mut()
        .filter_map(|r| {
            r.extract_record()
                .and_then(|record| record.into_untyped().ok())
        })
        .collect()
}

/// A reference is an enum that can be in two states: It either holds an ID to a record, or a
/// loaded record.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(untagged)]
pub enum Reference<T: Clone> {
    Id(String),
    Resolved(Record<T>),
}

impl<T: TypedValue> Reference<T> {
    /// Get the target id of the reference.
    pub fn id(&self) -> &str {
        match self {
            Self::Id(id) => id,
            Self::Resolved(record) => record.guid(),
        }
    }

    pub fn guid(&self) -> &str {
        match self {
            Self::Id(id) => id,
            Self::Resolved(record) => record.guid(),
        }
    }

    /// Get the loaded record if the reference is resolved.
    pub fn record(&self) -> Option<&Record<T>> {
        match self {
            Self::Resolved(record) => Some(record),
            Self::Id(_) => None,
        }
    }

    /// Get a mutable reference to the loaded record if the reference is resolved.
    pub fn record_mut(&mut self) -> Option<&mut Record<T>> {
        match self {
            Self::Resolved(record) => Some(record),
            Self::Id(_) => None,
        }
    }

    /// Get the loaded record if the reference is resolved, while consuming the reference.
    pub fn into_record(self) -> Option<Record<T>> {
        match self {
            Self::Resolved(record) => Some(record),
            Self::Id(_) => None,
        }
    }

    /// Extract the loaded record if the reference is resolved, while converting the reference to
    /// only hold an id.
    pub fn extract_record(&mut self) -> Option<Record<T>> {
        match self {
            Self::Resolved(_) => {
                let id = self.id().to_string();
                let next = Self::Id(id);
                let this = std::mem::replace(self, next);
                this.into_record()
            }
            Self::Id(_) => None,
        }
    }

    /// Check if the reference is resolved and contains a loaded record.
    pub fn resolved(&self) -> bool {
        matches!(self, Self::Resolved(_))
    }

    /// Resolve this reference with a [Resolver] if the reference is not yet in the resolved state.
    ///
    /// The [Resolver] trait is implemented on data stores.
    pub async fn resolve<R: Resolver>(&mut self, resolver: &R) -> Result<(), ResolveError> {
        if !self.resolved() {
            self.force_resolve(resolver).await?;
        }
        Ok(())
    }

    /// Resolve the record, even if the reference is already in the resolved state.
    pub async fn force_resolve<R: Resolver>(&mut self, resolver: &R) -> Result<(), ResolveError> {
        let record = resolver
            .resolve(self.id())
            .await
            .map_err(|err| ResolveError::new(self.id(), err))?;
        *self = Self::Resolved(record);
        Ok(())
    }

    /// Resolve the record while consuming self, and return a new reference in the resolved state.
    pub async fn into_resolved<R: Resolver>(mut self, resolver: &R) -> Result<Self, ResolveError> {
        self.resolve(resolver).await?;
        Ok(self)
    }

    /// Set the record to the resolved state while passing in the target record.
    pub fn set_resolved(&mut self, record: Record<T>) {
        *self = Self::Resolved(record)
    }
}
