use super::Rev;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::time;

use crate::Record;

#[derive(Debug)]
pub struct ChangesOpts {
    pub from: String,
    pub infinite: bool,
    pub batch_timeout: Option<time::Duration>,
    pub batch_limit: usize,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema, Clone)]
pub struct RecordBatch {
    pub records: Vec<Record>,
    pub rev: Rev,
}

impl RecordBatch {
    pub fn len(&self) -> usize {
        self.records.len()
    }
    pub fn rev(&self) -> &str {
        &self.rev
    }

    pub fn records(&self) -> &[Record] {
        &self.records[..]
    }

    pub fn into_inner(self) -> Vec<Record> {
        self.records
    }
}
