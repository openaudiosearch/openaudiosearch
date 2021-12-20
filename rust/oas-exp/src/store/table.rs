// use oas_common::{Resolvable, Resolver};
use std::marker::PhantomData;
use uuid::Uuid;

use crate::{Record, RecordValue};

use super::{RecordStore, TxResult, TxSuccess};

pub struct Table<T: RecordValue> {
    store: RecordStore,
    typ: PhantomData<T>,
}

impl<T> Table<T>
where
    T: RecordValue,
{
    pub fn new(store: RecordStore) -> Self {
        Self {
            store,
            typ: PhantomData,
        }
    }

    pub async fn get_one(&self, id: &Uuid) -> Result<Record, anyhow::Error> {
        let record = self.store.get_one(T::name(), &id).await?;
        let record = record.into_upcast::<T>()?;
        Ok(record)
    }

    pub async fn get_many(&self, ids: Vec<Uuid>) -> Result<Vec<Record>, anyhow::Error> {
        let ids = ids
            .into_iter()
            .map(|id| (T::name().to_string(), id))
            .collect();
        let records = self.store.get_many(ids).await?;
        let records = records
            .into_iter()
            .filter_map(|r| match r {
                Ok(r) => r.into_upcast::<T>().ok(),
                Err(_e) => None,
            })
            .collect();
        Ok(records)
    }

    // pub async fn get_many_resolved(&self, ids: Vec<Guid>) -> Result<Vec<Record>, anyhow::Error>
    // where
    //     T: Resolvable + Send,
    // {
    //     let mut records = self.get_many(ids).await?;
    //     self.store.resolve_all_refs(&mut records).await?;
    //     Ok(records)
    // }

    pub async fn put_one(&self, record: Record) -> Result<TxSuccess, anyhow::Error> {
        self.store.put_one(record).await
    }

    pub async fn put_many(&self, records: Vec<Record>) -> Result<Vec<TxResult>, anyhow::Error> {
        let res = self.store.put_many(records).await?;
        Ok(res)
    }
}
