use crate::reference::Reference;
use crate::{Record, TypedValue, UntypedRecord};
use std::{fmt, marker::PhantomData};

#[async_trait::async_trait]
pub trait Resolvable: TypedValue {
    async fn resolve_refs<R: Resolver + Send + Sync>(
        &mut self,
        _resolver: &R,
    ) -> Result<(), MissingRefsError> {
        Ok(())
    }

    fn extract_refs(&mut self) -> Vec<UntypedRecord> {
        vec![]
    }
}

#[async_trait::async_trait]
pub trait Resolver {
    type Error: std::error::Error + Send + Sync + 'static;

    async fn resolve<T: TypedValue>(&self, id: &str) -> Result<Record<T>, Self::Error>;

    async fn resolve_all<T: TypedValue + Send>(
        &self,
        ids: &[&str],
    ) -> Vec<Result<Record<T>, Self::Error>> {
        let futs: Vec<_> = ids.iter().map(|id| self.resolve(id)).collect();
        let results = futures_util::future::join_all(futs).await;
        results
    }

    async fn resolve_refs<T: TypedValue + Send>(
        &self,
        references: &mut [Reference<T>],
    ) -> Result<(), MissingRefsError> {
        let unresolved_refs: Vec<(usize, String)> = references
            .iter()
            .enumerate()
            .filter_map(|(i, r)| match r {
                Reference::Id(id) => Some((i, id.clone())),
                _ => None,
            })
            .collect();
        let unresolved_ids: Vec<&str> = unresolved_refs.iter().map(|(_, id)| id.as_str()).collect();
        let results = self.resolve_all(&unresolved_ids).await;
        let mut errs: Vec<ResolveError> = vec![];
        for (i, result) in results.into_iter().enumerate() {
            match result {
                Ok(record) => references.get_mut(i).unwrap().set_resolved(record),
                Err(err) => errs.push(ResolveError::new(
                    references.get(i).unwrap().id(),
                    err.into(),
                )),
            }
        }
        match errs.len() {
            0 => Ok(()),
            _ => Err(MissingRefsError(errs)),
        }
    }
}

#[derive(Debug)]
pub struct ResolveError {
    id: String,
    error: anyhow::Error,
    // typ: String,
}
// pub struct ResolveError {
//     id: String,
//     error: anyhow::Error,
//     typ: PhantomData<T>,
// }
//

#[derive(Debug)]
pub struct MissingRefsError(pub Vec<ResolveError>);

impl fmt::Display for MissingRefsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Failed to resolve {} refs", self.0.len())
    }
}

impl std::error::Error for MissingRefsError {}

impl ResolveError {
    pub fn new(id: &str, error: anyhow::Error) -> Self {
        Self {
            id: id.to_string(),
            error,
            // typ: PhantomData
        }
    }

    pub fn into_reference<T: Clone>(self) -> Reference<T> {
        Reference::Id(self.id)
    }
}

impl<T: TypedValue> From<ResolveError> for Reference<T> {
    fn from(err: ResolveError) -> Self {
        Reference::Id(err.id)
    }
}

impl std::error::Error for ResolveError {}
impl fmt::Display for ResolveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Failed to resolve {}: {}", self.id, self.error)
    }
}
