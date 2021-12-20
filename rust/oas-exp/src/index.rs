use crate::Record;
use std::collections::HashMap;

pub type StateBuf = Vec<u8>;
pub type Config = HashMap<String, serde_json::Value>;
pub type Query = serde_json::Value;

pub struct Batch {
    records: Vec<Record>,
}

pub struct QueryOpts {
    limit: usize,
    offset: usize,
}

pub struct QueryBatch {
    records: Vec<Record>,
    total: Option<usize>,
    offset: usize,
}

struct IndexInfo {
    indexed: u64,
}

struct IndexId(String);

#[async_trait::async_trait]
trait Index {
    async fn open(&self, id: IndexId);
    async fn configure(&mut self, config: Config, previous: Option<Config>) -> anyhow::Result<()>;
    async fn ingest(&self, batch: Batch) -> anyhow::Result<()>;
    async fn info(&self) -> IndexInfo;
    async fn query(&self, opts: QueryOpts, query: Query) -> anyhow::Result<QueryBatch>;
}
