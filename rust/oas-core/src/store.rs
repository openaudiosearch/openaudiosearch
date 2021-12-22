use async_trait::async_trait;
use futures::stream::Stream;
use oas_common::{Guid, Record, Resolver, TypedValue, UntypedRecord};
use std::pin::Pin;
use std::str::FromStr;
use std::sync::Arc;

pub mod structs;
pub mod table;
mod transaction;
pub use structs::{ChangesOpts, UntypedRecordBatch};
pub use table::Table;
pub use transaction::{Op, Rev, Transaction, TxError, TxResult, TxSuccess};

#[derive(thiserror::Error, Debug)]
pub enum StorageError {
    #[error("IO Error")]
    IO(#[from] std::io::Error),
    #[error("HTTP Error")]
    Http(#[from] reqwest::Error),
    #[error("Other")]
    Other(#[from] Box<dyn std::error::Error>),
}

#[async_trait]
pub trait Storage: std::fmt::Debug {
    // type Error: std::error::Error;
    // type Changes: Stream<Item = UntypedRecordBatch>;
    async fn commit(&self, tx: Transaction) -> Result<Vec<TxResult>, anyhow::Error>;
    async fn get_one(&self, id: &Guid) -> Result<UntypedRecord, anyhow::Error>;
    async fn get_many(
        &self,
        ids: Vec<Guid>,
    ) -> Result<Vec<Result<UntypedRecord, anyhow::Error>>, anyhow::Error>;
    // async fn changes(&mut self, opts: ChangesOpts) -> Self::Changes;
    async fn changes(&self, opts: ChangesOpts) -> Pin<Box<dyn Stream<Item = UntypedRecordBatch>>>;
}

pub enum StorageRequest {
    Transaction(Transaction),
    GetOne(Guid),
    GetMany(Vec<Guid>),
    Changes(ChangesOpts),
    History(Guid),
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

    pub fn table<T: TypedValue>(&self) -> Table<T> {
        Table::new(self.clone())
    }

    pub async fn get_one(&self, id: &Guid) -> Result<UntypedRecord, anyhow::Error> {
        let record = self.inner.store.get_one(id).await?;
        Ok(record)
    }

    pub async fn get_many(
        &self,
        ids: Vec<Guid>,
    ) -> Result<Vec<Result<UntypedRecord, anyhow::Error>>, anyhow::Error> {
        let records = self.inner.store.get_many(ids).await?;
        Ok(records)
    }

    pub async fn put_one(&self, record: UntypedRecord) -> Result<TxSuccess, anyhow::Error> {
        let mut tx = Transaction::new();
        tx.put_untyped(record);
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

    pub async fn put_many(
        &self,
        records: Vec<UntypedRecord>,
    ) -> Result<Vec<TxResult>, anyhow::Error> {
        let mut tx = Transaction::new();
        for record in records.into_iter() {
            tx.put_untyped(record);
        }
        let res = self.inner.store.commit(tx).await?;
        Ok(res)
    }

    pub async fn changes(
        &self,
        opts: ChangesOpts,
    ) -> Pin<Box<dyn Stream<Item = UntypedRecordBatch>>> {
        self.inner.store.changes(opts).await
    }
}

#[async_trait::async_trait]
impl Resolver for RecordStore {
    /// Resolve (load) a single record by its id.
    async fn resolve<T: TypedValue>(&self, id: &str) -> Result<Record<T>, anyhow::Error> {
        let id = Guid::from_str(id)?;
        let record = self.get_one(&id).await?;
        let record = record.into_typed::<T>()?;
        Ok(record)
    }
    async fn resolve_all<T: TypedValue + Send>(
        &self,
        ids: &[&str],
    ) -> Vec<Result<Record<T>, anyhow::Error>> {
        let ids: Vec<Guid> = ids
            .iter()
            .filter_map(|id| Guid::from_str(id).ok())
            .collect();
        match self.get_many(ids).await {
            Ok(records) => records
                .into_iter()
                .map(|r| r.map(|r| r.into_typed::<T>().map_err(|e| e.into())))
                .flatten()
                .collect(),
            Err(_) => vec![],
        }
    }
}
