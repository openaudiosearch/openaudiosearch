use oas_common::{EncodingError, Guid, Record, TypedValue, UntypedRecord};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt;

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
    pub id: String, // TODO: Guid
    pub rev: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema, thiserror::Error)]
pub struct TxError {
    #[serde(default)]
    pub ok: bool,
    pub error: String,
    pub id: Option<String>, // TODO: Guid
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
    Put(UntypedRecord),
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
pub struct Transaction {
    puts: Vec<UntypedRecord>,
    dels: Vec<Guid>,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
pub struct TransactionOps {
    pub puts: Vec<UntypedRecord>,
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
    pub fn put_untyped(&mut self, record: UntypedRecord) {
        self.op(Op::Put(record))
    }

    pub fn put<T: TypedValue>(&mut self, record: Record<T>) -> Result<(), EncodingError> {
        let record = record.into_untyped()?;
        self.op(Op::Put(record));
        Ok(())
    }

    pub fn del(&mut self, guid: Guid) {
        self.op(Op::Delete(guid))
    }

    pub fn into_ops(self) -> TransactionOps {
        TransactionOps {
            puts: self.puts,
            dels: self.dels,
        }
    }
}
