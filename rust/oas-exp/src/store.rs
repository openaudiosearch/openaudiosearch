use async_trait::async_trait;
use futures::stream::Stream;
use uuid::Uuid;
// use oas_common::{Guid, Record, Resolver, TypedValue, UntypedRecord};
use std::pin::Pin;
use std::str::FromStr;
use std::sync::Arc;

pub mod structs;
pub mod table;
mod transaction;
pub use structs::{ChangesOpts, RecordBatch};
pub use table::Table;
pub use transaction::{Op, Rev, Transaction, TxError, TxResult, TxSuccess};

use crate::{RawRecord, Record, RecordValue};

#[derive(thiserror::Error, Debug)]
pub enum StorageError {
    #[error("IO Error")]
    IO(#[from] std::io::Error),
    #[error("HTTP Error")]
    Http(#[from] reqwest::Error),
    #[error("Other")]
    Other(#[from] Box<dyn std::error::Error>),
}

// pub trait AsRecordPtr {
//     fn typ(&self) -> &str;
//     fn id(&self) -> &Uuid;
// }
// impl AsRecordPtr for (String, Uuid) {
//     fn typ(&self) -> &str {
//         &self.0
//     }
//     fn id(&self) -> &Uuid {
//         &self.1
//     }
// }
// impl AsRecordPtr for (&str, &Uuid) {
//     fn typ(&self) -> &str {
//         &self.0
//     }
//     fn id(&self) -> &Uuid {
//         &self.1
//     }
// }
// impl AsRecordPtr for Record {
//     fn typ(&self) -> &str {
//         self.typ()
//     }
//     fn id(&self) -> &Uuid {
//         self.id()
//     }
// }

// pub struct RecordPtr {
//     id: Uuid,
//     typ: String,
//     record: Option<Record>,
// }
// impl AsRecordPtr for RecordPtr {
//     fn typ(&self) -> &str {
//         &self.typ
//     }
//     fn id(&self) -> &Uuid {
//         &self.id
//     }
// }
// pub struct TypedRecordPtr<T> {
//     id: Uuid,
//     record: Option<Record>,
//     typ: PhantomData<T>,
// }
// impl<T> AsRecordPtr for TypedRecordPtr<T>
// where
//     T: RecordValue,
// {
//     fn typ(&self) -> &str {
//         T::name()
//     }
//     fn id(&self) -> &Uuid {
//         &self.id
//     }
// }

pub struct FetchAllOpts {
    offset: Option<usize>,
    limit: Option<usize>,
}
pub struct FetchAllResponse {
    records: Vec<RawRecord>,
    total: Option<usize>,
}

#[async_trait]
pub trait Storage: std::fmt::Debug {
    // type Error: std::error::Error;
    // type Changes: Stream<Item = RecordBatch>;
    async fn commit(&self, tx: Transaction) -> Result<Vec<TxResult>, anyhow::Error>;
    async fn fetch_one(&self, typ: &str, id: &Uuid) -> Result<RawRecord, anyhow::Error>;
    async fn fetch_many(
        &self,
        guids: Vec<(String, Uuid)>,
    ) -> Result<Vec<Result<RawRecord, anyhow::Error>>, anyhow::Error>;
    async fn fetch_all(
        &self,
        typ: &str,
        opts: FetchAllOpts,
    ) -> anyhow::Result<FetchAllResponse, anyhow::Error>;
    // async fn changes(&mut self, opts: ChangesOpts) -> Self::Changes;
    async fn changes(&self, opts: ChangesOpts) -> Pin<Box<dyn Stream<Item = RecordBatch>>>;
}

pub enum StorageRequest {
    Transaction(Transaction),
    GetOne(Uuid),
    GetMany(Vec<Uuid>),
    Changes(ChangesOpts),
    History(Uuid),
}

// type AnyStorage=Storage<Changes = Pin<Box<dyn

#[async_trait]
pub trait ChangesStream {}

#[derive(Clone, Debug)]
pub struct RecordStore {
    inner: Arc<RecordStoreInner>,
}

#[derive(Debug)]
pub struct RecordStoreInner {
    store: Box<dyn Storage + Send + Sync>,
}

// impl<T> RecordStore<T> where T: Storage {}
impl RecordStore {
    pub fn with_storage(storage: Box<dyn Storage + Send + Sync>) -> Self {
        let inner = RecordStoreInner { store: storage };
        Self {
            inner: Arc::new(inner),
        }
    }

    pub fn table<T: RecordValue>(&self) -> Table<T> {
        Table::new(self.clone())
    }

    pub async fn get_one(&self, typ: &str, id: &Uuid) -> Result<Record, anyhow::Error> {
        let record = self.inner.store.fetch_one(typ, id).await?;
        let record = Record::from_raw(record);
        Ok(record)
    }

    pub async fn get_many(
        &self,
        guids: Vec<(String, Uuid)>,
    ) -> Result<Vec<Result<Record, anyhow::Error>>, anyhow::Error> {
        let records = self.inner.store.fetch_many(guids).await?;
        let records = records
            .into_iter()
            .map(|r| r.map(|r| Record::from_raw(r)))
            .collect();
        Ok(records)
    }

    pub async fn put_one(&self, record: Record) -> Result<TxSuccess, anyhow::Error> {
        let mut tx = Transaction::new();
        tx.put(record);
        let res = self.inner.store.commit(tx).await?;
        let res = res
            .into_iter()
            .nth(0)
            .ok_or_else(|| anyhow::anyhow!("Transaction produced no results"))?;
        match res {
            TxResult::Ok(res) => Ok(res),
            TxResult::Err(err) => Err(err.into()),
        }
    }

    pub async fn put_many(&self, records: Vec<Record>) -> Result<Vec<TxResult>, anyhow::Error> {
        let mut tx = Transaction::new();
        for record in records.into_iter() {
            tx.put(record);
        }
        let res = self.inner.store.commit(tx).await?;
        Ok(res)
    }

    // pub async fn changes(
    //     &self,
    //     opts: ChangesOpts,
    // ) -> Pin<Box<dyn Stream<Item = RecordBatch>>> {
    //     self.inner.store.changes(opts).await
    // }
}

// #[async_trait::async_trait]
// impl Resolver for RecordStore {
//     /// Resolve (load) a single record by its id.
//     async fn resolve<T: TypedValue>(&self, id: &str) -> Result<Record<T>, anyhow::Error> {
//         let id = Uuid::from_str(id)?;
//         let record = self.get_one(&id).await?;
//         let record = record.into_typed::<T>()?;
//         Ok(record)
//     }
//     async fn resolve_all<T: TypedValue + Send>(
//         &self,
//         ids: &[&str],
//     ) -> Vec<Result<Record<T>, anyhow::Error>> {
//         let ids: Vec<Uuid> = ids
//             .iter()
//             .filter_map(|id| Uuid::from_str(id).ok())
//             .collect();
//         match self.get_many(ids).await {
//             Ok(records) => records
//                 .into_iter()
//                 .map(|r| r.map(|r| r.into_typed::<T>().map_err(|e| e.into())))
//                 .flatten()
//                 .collect(),
//             Err(_) => vec![],
//         }
//     }
// }
