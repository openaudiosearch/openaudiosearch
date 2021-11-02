use futures::{Stream, StreamExt};
use oas_common::{Record, TypedValue};
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use tracing::debug;

use crate::couch::changes::BatchOpts;

use super::{changes::UntypedRecordBatch, CouchDB};

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct DurablePointer {
    seq: String,
}

impl TypedValue for DurablePointer {
    const NAME: &'static str = "meta.DurableChanges";
}

pub struct ChangesOpts {
    pub infinite: bool,
}
impl Default for ChangesOpts {
    fn default() -> Self {
        Self { infinite: true }
    }
}

pub struct DurableChanges {
    id: String,
    needs_ack: bool,
    meta: CouchDB,
    main: CouchDB,
    changes: Option<Pin<Box<dyn Stream<Item = UntypedRecordBatch> + Unpin + Send + 'static>>>,
    seq: Option<String>,
    init: bool,
    opts: ChangesOpts,
}

impl DurableChanges {
    pub async fn new(main: CouchDB, meta: CouchDB, id: String, opts: ChangesOpts) -> Self {
        Self {
            id,
            meta,
            main,
            changes: None,
            seq: None,
            needs_ack: false,
            init: false,
            opts,
        }
    }

    async fn init(&mut self) {
        let seq = self
            .meta
            .table::<DurablePointer>()
            .get(&self.id)
            .await
            .map(|record| record.value.seq);
        match seq {
            Ok(seq) => {
                debug!("Init durable changes for `{}` at {}", self.id, seq);
                self.seq = Some(seq);
            }
            Err(err) => {
                debug!(error = %err, "Init durable changes for `{}`, no previous seq", self.id);
            }
        }

        let mut changes = self.main.changes(self.seq.clone());
        changes.set_infinite(self.opts.infinite);
        let mut batch_opts = BatchOpts::default();
        batch_opts.max_len = 3;
        let changes = changes.batched_untyped_records(batch_opts);
        let changes: Box<dyn Stream<Item = _> + Unpin + Send + 'static> = Box::new(changes);
        let changes = Pin::new(changes);
        self.changes = Some(changes);
        self.init = true;
    }

    pub async fn ack(&mut self) -> anyhow::Result<()> {
        if let Some(seq) = &self.seq {
            let seq = seq.clone();
            let record = Record::from_id_and_value(self.id.clone(), DurablePointer { seq });
            self.meta.table::<DurablePointer>().put(record).await?;
            self.needs_ack = false;
        }
        Ok(())
    }

    pub async fn next(&mut self) -> anyhow::Result<Option<UntypedRecordBatch>> {
        if !self.init {
            self.init().await;
        }
        if self.needs_ack {
            self.ack().await?;
        }
        let batch = self.changes.as_mut().unwrap().next().await;
        if let Some(batch) = &batch {
            self.seq = batch.last_seq().as_ref().map(|s| s.to_string());
            self.needs_ack = true;
        }
        Ok(batch)
    }
}
