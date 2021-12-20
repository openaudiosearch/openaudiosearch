// use oas_common::{EncodingError, Guid, Record, TypedValue, UntypedRecord};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::{Guid, Record, RecordValue, Uuid};

pub type Rev = String;

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(untagged)]
pub enum TxResult {
    Ok(TxSuccess),
    Err(TxError),
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
pub struct TxSuccess {
    pub ok: bool,
    pub id: Guid,
    pub rev: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema, thiserror::Error)]
pub struct TxError {
    #[serde(default)]
    pub ok: bool,
    pub error: String,
    pub id: Option<Guid>, // TODO: Guid
    pub reason: String,
}

impl fmt::Display for TxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.error, self.reason)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(tag = "type")]
pub enum Op {
    Delete(Guid),
    Put(Record),
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
pub struct Transaction {
    puts: Vec<Record>,
    dels: Vec<Guid>,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
pub struct TransactionOps {
    pub puts: Vec<Record>,
    pub dels: Vec<Guid>,
}

impl Transaction {
    pub fn new() -> Self {
        Self {
            puts: vec![],
            dels: vec![],
        }
    }
    pub fn op(&mut self, op: Op) {
        match op {
            Op::Delete(guid) => self.dels.push(guid),
            Op::Put(record) => self.puts.push(record),
        }
    }
    pub fn put(&mut self, record: Record) -> &mut Self {
        self.op(Op::Put(record));
        self
    }

    pub fn del(&mut self, guid: Guid) -> &mut Self {
        self.op(Op::Delete(guid));
        self
    }

    pub fn into_ops(self) -> TransactionOps {
        TransactionOps {
            puts: self.puts,
            dels: self.dels,
        }
    }
}
