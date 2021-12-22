use super::Rev;
use oas_common::{Guid, UntypedRecord};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::time;

#[derive(Debug)]
pub struct ChangesOpts {
    pub from: String,
    pub infinite: bool,
    pub batch_timeout: Option<time::Duration>,
    pub batch_limit: usize,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema, Clone)]
pub struct UntypedRecordBatch {
    pub records: Vec<UntypedRecord>,
    pub rev: Rev,
}

impl UntypedRecordBatch {
    pub fn len(&self) -> usize {
        self.records.len()
    }
    pub fn rev(&self) -> &str {
        &self.rev
    }

    pub fn records(&self) -> &[UntypedRecord] {
        &self.records[..]
    }

    pub fn into_inner(self) -> Vec<UntypedRecord> {
        self.records
    }
}
