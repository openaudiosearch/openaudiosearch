use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use oas_common::{DecodingError, Record, TypedValue, UntypedRecord};

pub type Object = serde_json::Map<String, serde_json::Value>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocMeta {
    #[serde(rename = "_id")]
    id: String,
    #[serde(skip_serializing_if = "is_null")]
    #[serde(rename = "_rev")]
    rev: Option<String>,
}

impl DocMeta {
    pub fn new(id: String, rev: Option<String>) -> Self {
        Self { id, rev }
    }
    pub fn with_id(id: String) -> Self {
        Self { id, rev: None }
    }
    pub fn with_id_and_rev(id: String, rev: String) -> Self {
        Self { id, rev: Some(rev) }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocList {
    pub total_rows: u32,
    pub offset: u32,
    pub rows: Vec<DocListEntry>,
}

impl DocList {
    pub fn rows(&self) -> impl Iterator<Item = &Doc> {
        self.rows[..].iter().map(|row| &row.doc)
    }

    pub fn into_typed_records<T>(self) -> Vec<std::result::Result<Record<T>, DecodingError>>
    where
        T: TypedValue,
    {
        self.rows
            .into_iter()
            .map(|row| row.doc.into_typed_record())
            .collect()
    }

    pub fn into_typed_docs<T: DeserializeOwned>(self) -> serde_json::Result<Vec<T>> {
        let results: serde_json::Result<Vec<T>> = self
            .rows
            .into_iter()
            .map(|d| d.doc.into_typed::<T>())
            .collect();
        results
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocListEntry {
    pub id: String,
    pub key: String,
    pub doc: Doc,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Doc {
    #[serde(flatten)]
    pub meta: DocMeta,
    #[serde(flatten)]
    pub doc: Object,
}

impl Doc {
    pub fn into_typed<T: DeserializeOwned>(self) -> serde_json::Result<T> {
        let doc: T = serde_json::from_value(serde_json::Value::Object(self.doc))?;
        Ok(doc)
    }

    pub fn from_typed<T>(meta: DocMeta, doc: T) -> anyhow::Result<Self>
    where
        T: Serialize,
    {
        let doc = serde_json::to_value(doc)?;
        if let serde_json::Value::Object(doc) = doc {
            Ok(Self::new(meta, doc))
        } else {
            Err(anyhow::anyhow!("Not an object"))
        }
    }

    pub fn from_typed_record<T>(record: Record<T>) -> Self
    where
        T: TypedValue,
    {
        let meta = DocMeta::with_id(record.guid().to_string());
        // TODO: Remove unwrap
        let doc = record.into_json_object().unwrap();
        Self { meta, doc }
    }

    pub fn from_untyped_record(record: UntypedRecord) -> Self {
        let meta = DocMeta::with_id(record.guid().to_string());
        // TODO: Remove unwrap
        let doc = record.into_json_object().unwrap();
        Self { meta, doc }
    }

    pub fn into_untyped_record(self) -> serde_json::Result<UntypedRecord> {
        let record: UntypedRecord = serde_json::from_value(serde_json::Value::Object(self.doc))?;
        Ok(record)
    }

    pub fn into_typed_record<T>(self) -> std::result::Result<Record<T>, DecodingError>
    where
        T: TypedValue,
    {
        let record: UntypedRecord = serde_json::from_value(serde_json::Value::Object(self.doc))?;
        let record: Record<T> = record.into_typed_record()?;
        Ok(record)
    }

    pub fn new(meta: DocMeta, doc: Object) -> Self {
        Self { meta, doc }
    }

    pub fn id(&self) -> &str {
        &self.meta.id
    }
    pub fn rev(&self) -> Option<&str> {
        self.meta.rev.as_deref()
    }
    pub fn set_rev(&mut self, rev: Option<String>) {
        self.meta.rev = rev;
    }

    pub fn is_first_rev(&self) -> Option<bool> {
        self.rev().map(|rev| rev.starts_with("1-"))
    }
}

impl<T> From<Record<T>> for Doc
where
    T: TypedValue,
{
    fn from(record: Record<T>) -> Self {
        Doc::from_typed_record(record)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Event {
    Change(ChangeEvent),
    Finished(FinishedEvent),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChangeEvent {
    pub seq: String,
    pub id: String,
    pub changes: Vec<Change>,

    #[serde(default)]
    pub deleted: bool,

    #[serde(default)]
    pub doc: Option<Doc>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Change {
    pub rev: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FinishedEvent {
    pub last_seq: String,
    pub pending: Option<u64>, // not available on CouchDB 1.0
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetChangesResult {
    pub results: Vec<ChangeEvent>,
    pub last_seq: String,
    pub pending: Option<i64>,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
pub struct PutResponse {
    pub id: String,
    pub ok: bool,
    pub rev: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ErrorDetails {
    pub error: String,
    pub id: Option<String>,
    pub reason: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum PutResult {
    Ok(PutResponse),
    Err(ErrorDetails),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BulkGetResponse {
    pub results: Vec<BulkGetItem>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BulkGetItem {
    pub id: String,
    pub docs: Vec<DocResult>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum DocResult {
    #[serde(rename = "ok")]
    Ok(Doc),
    #[serde(rename = "error")]
    Err(ErrorDetails),
}

fn is_null<T: Serialize>(t: &T) -> bool {
    serde_json::to_value(t)
        .unwrap_or(serde_json::Value::Null)
        .is_null()
}
