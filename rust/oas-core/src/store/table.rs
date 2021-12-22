// use super::changes::RecordChangesStream;
// use super::{CouchDB, CouchResult, PutResponse, PutResult};
use oas_common::{EncodingError, Guid, Record, TypedValue, UntypedRecord};
use oas_common::{Resolvable, Resolver};
use std::marker::PhantomData;

use super::{RecordStore, TxResult, TxSuccess};

pub struct Table<T: TypedValue> {
    store: RecordStore,
    typ: PhantomData<T>,
}

impl<T> Table<T>
where
    T: TypedValue,
{
    pub fn new(store: RecordStore) -> Self {
        Self {
            store,
            typ: PhantomData,
        }
    }

    pub async fn get_one(&self, id: &Guid) -> Result<Record<T>, anyhow::Error> {
        let record = self.store.get_one(&id).await?;
        let record = record.into_typed::<T>()?;
        Ok(record)
    }

    pub async fn get_many(&self, ids: Vec<Guid>) -> Result<Vec<Record<T>>, anyhow::Error> {
        let records = self.store.get_many(ids).await?;
        let records = records
            .into_iter()
            .filter_map(|r| match r {
                Ok(r) => r.into_typed::<T>().ok(),
                Err(_e) => None,
            })
            .collect();
        Ok(records)
    }

    pub async fn get_many_resolved(&self, ids: Vec<Guid>) -> Result<Vec<Record<T>>, anyhow::Error>
    where
        T: Resolvable + Send,
    {
        let mut records = self.get_many(ids).await?;
        self.store.resolve_all_refs(&mut records).await?;
        Ok(records)
    }

    pub async fn put_one(&self, record: Record<T>) -> Result<TxSuccess, anyhow::Error> {
        let record = record.into_untyped()?;
        self.store.put_one(record).await
    }

    pub async fn put_many(&self, records: Vec<Record<T>>) -> Result<Vec<TxResult>, anyhow::Error> {
        let records = records
            .into_iter()
            .map(|r| r.into_untyped())
            .collect::<Result<Vec<UntypedRecord>, EncodingError>>()?;
        let res = self.store.put_many(records).await?;
        Ok(res)
    }
}
