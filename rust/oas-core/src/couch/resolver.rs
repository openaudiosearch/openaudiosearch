use super::{CouchDB, CouchError};
use oas_common::reference::Resolver;
use oas_common::{Record, TypedValue};

#[async_trait::async_trait]
impl Resolver for CouchDB {
    type Error = CouchError;
    async fn resolve<T: TypedValue>(&self, id: &str) -> Result<Record<T>, Self::Error> {
        let doc = self.get_doc(id).await?;
        let record = doc.into_typed_record::<T>()?;
        Ok(record)
    }
}

#[async_trait::async_trait]
impl Resolver for &CouchDB {
    type Error = CouchError;
    async fn resolve<T: TypedValue>(&self, id: &str) -> Result<Record<T>, Self::Error> {
        <CouchDB as Resolver>::resolve(self, id).await
    }
}
