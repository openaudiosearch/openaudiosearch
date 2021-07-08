use crate::{Record, TypedValue, UntypedRecord};
use serde::{Deserialize, Serialize};

pub use crate::resolver::{ResolveError, Resolver};

pub fn extract_refs<T: TypedValue>(refs: &mut [Reference<T>]) -> Vec<UntypedRecord> {
    refs.iter_mut()
        .filter_map(|r| {
            r.extract_record()
                .and_then(|record| record.into_untyped_record().ok())
        })
        .collect()
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Reference<T: Clone> {
    Id(String),
    Resolved(Record<T>),
}

impl<T: TypedValue> Reference<T> {
    pub fn id(&self) -> &str {
        match self {
            Self::Id(id) => &id,
            Self::Resolved(record) => record.guid(),
        }
    }

    pub fn record(&self) -> Option<&Record<T>> {
        match self {
            Self::Resolved(record) => Some(&record),
            Self::Id(_) => None,
        }
    }

    pub fn record_mut(&mut self) -> Option<&mut Record<T>> {
        match self {
            Self::Resolved(record) => Some(record),
            Self::Id(_) => None,
        }
    }

    pub fn into_record(self) -> Option<Record<T>> {
        match self {
            Self::Resolved(record) => Some(record),
            Self::Id(_) => None,
        }
    }

    pub fn extract_record(&mut self) -> Option<Record<T>> {
        match self {
            Self::Resolved(_) => {
                let id = self.id().to_string();
                let next = Self::Id(id);
                let this = std::mem::replace(self, next);
                let record = this.into_record();
                record
            }
            Self::Id(_) => None,
        }
    }

    pub fn resolved(&self) -> bool {
        matches!(self, Self::Resolved(_))
    }

    pub async fn resolve<R: Resolver>(&mut self, resolver: &R) -> Result<(), ResolveError> {
        if !self.resolved() {
            self.force_resolve(resolver).await?;
        }
        Ok(())
    }

    pub async fn force_resolve<R: Resolver>(&mut self, resolver: &R) -> Result<(), ResolveError> {
        let record = resolver
            .resolve(self.id())
            .await
            .map_err(|err| ResolveError::new(self.id(), anyhow::Error::new(err)))?;
        *self = Self::Resolved(record);
        Ok(())
    }

    pub async fn into_resolved<R: Resolver>(mut self, resolver: &R) -> Result<Self, ResolveError> {
        self.resolve(resolver).await?;
        Ok(self)
    }

    pub fn set_resolved(&mut self, record: Record<T>) {
        *self = Self::Resolved(record)
    }
}
