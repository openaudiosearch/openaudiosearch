use super::{ChangesStream, CouchDB, CouchError};
use async_trait::async_trait;
use futures::stream::{Stream, StreamExt};
use oas_common::{Guid, Record, TypedValue, UntypedRecord};
use std::pin::Pin;

use crate::store::{
    ChangesOpts, Rev, Storage, StorageError, Transaction, TxResult, UntypedRecordBatch,
};

#[async_trait]
impl Storage for CouchDB {
    async fn commit(&self, tx: Transaction) -> Result<Vec<TxResult>, anyhow::Error> {
        let ops = tx.into_ops();
        self.put_untyped_record_bulk_update(ops.puts).await?;
        let del_ids = ops.dels.into_iter().map(|id| id.to_string()).collect();
        self.delete_bulk_update(del_ids).await?;
        Ok(vec![])
    }
    async fn get_one(&self, id: &Guid) -> Result<UntypedRecord, anyhow::Error> {
        self.get_record_untyped(id.as_str())
            .await
            .map_err(|e| e.into())
    }
    async fn get_many(
        &self,
        ids: Vec<Guid>,
    ) -> Result<Vec<Result<UntypedRecord, anyhow::Error>>, anyhow::Error> {
        let ids: Vec<&str> = ids.iter().map(|id| id.as_str()).collect();
        let records = self.get_many_records_untyped(&ids[..]).await?;
        let results: Vec<Result<UntypedRecord, anyhow::Error>> =
            records.into_iter().map(|r| Ok(r)).collect();
        Ok(results)
    }
    async fn changes(&self, opts: ChangesOpts) -> Pin<Box<dyn Stream<Item = UntypedRecordBatch>>> {
        let mut changes = CouchDB::changes(&self, Some(opts.from));
        changes.set_infinite(opts.infinite);
        let batch_opts = super::changes::BatchOpts {
            timeout: opts.batch_timeout.unwrap_or_default(),
            max_len: opts.batch_limit,
        };
        let changes = changes.batched_untyped_records(&batch_opts);
        let changes = changes.map(|batch| UntypedRecordBatch {
            records: batch.records,
            rev: batch.last_seq.unwrap(),
        });
        let changes: Pin<Box<dyn Stream<Item = UntypedRecordBatch>>> = Box::pin(changes);
        changes
    }
}

// use async_trait::async_trait;

// #[async_trait]
// trait Store {
// }
