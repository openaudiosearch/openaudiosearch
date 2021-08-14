// use super::changes::RecordChangesStream;
use super::{CouchDB, CouchResult, PutResponse, PutResult};
use oas_common::{Record, TypedValue};
use oas_common::{Resolvable, Resolver};
use std::marker::PhantomData;

pub struct Table<T: TypedValue> {
    db: CouchDB,
    typ: PhantomData<T>,
}

impl<T> Table<T>
where
    T: TypedValue,
{
    pub fn new(db: CouchDB) -> Self {
        Self {
            db,
            typ: PhantomData,
        }
    }

    // pub fn changes(&self, last_seq: Option<String>) -> RecordChangesStream<T> {
    //     self.db.changes(last_seq).batched_records()
    // }

    pub async fn get(&self, id: &str) -> CouchResult<Record<T>> {
        self.db.get_record(&T::guid(id)).await
    }

    pub async fn get_bulk(&self, ids: &[&str]) -> CouchResult<Vec<Record<T>>> {
        let ids: Vec<String> = ids.iter().map(|id| T::guid(id)).collect();
        let ids: Vec<&str> = ids.iter().map(|id| id.as_str()).collect();
        let records = self.db.get_many_records::<T>(&ids).await?;
        Ok(records)
    }

    pub async fn get_bulk_resolved(&self, ids: &[&str]) -> CouchResult<Vec<Record<T>>>
    where
        T: Resolvable + Send,
    {
        let mut records = self.get_bulk(&ids).await?;
        self.db.resolve_all_refs(&mut records).await?;
        Ok(records)
    }

    pub async fn get_all(&self) -> CouchResult<Vec<Record<T>>> {
        let records = self.db.get_all_records::<T>().await?;
        Ok(records)
    }

    pub async fn put(&self, record: Record<T>) -> CouchResult<PutResponse> {
        self.db.put_record(record).await
    }

    pub async fn put_bulk(&self, records: Vec<Record<T>>) -> CouchResult<Vec<PutResult>> {
        self.db.put_record_bulk(records).await
    }
}
