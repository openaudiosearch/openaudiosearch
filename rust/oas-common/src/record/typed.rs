use schemars::JsonSchema;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use super::error::{EncodingError, ValidationError};
use super::{RecordMeta, TypedValue, UntypedRecord};
use crate::{ElasticMapping, JsonObject, MissingRefsError, Resolvable, Resolver};

/// A record with a strongly typed value.
///
/// All values should implement [TypedValue].
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
pub struct TypedRecord<T>
where
    T: Clone,
{
    #[serde(rename = "$meta")]
    pub meta: RecordMeta,
    #[serde(flatten)]
    pub value: T,
}

impl<T> TypedRecord<T>
where
    T: TypedValue,
{
    /// Get the guid of the record.
    pub fn guid(&self) -> &str {
        &self.meta.guid
    }

    /// Get the id of the record.
    pub fn id(&self) -> &str {
        &self.meta.id
    }

    /// Get the typ of the record.
    pub fn typ(&self) -> &str {
        &self.meta.typ
    }

    // Get a reference to the record meta.
    pub fn meta(&self) -> &RecordMeta {
        &self.meta
    }

    // Get a mutable reference to the record meta.
    pub fn meta_mut(&mut self) -> &mut RecordMeta {
        &mut self.meta
    }

    /// Create a new record from an id and a value.
    pub fn from_id_and_value(id: impl ToString, value: T) -> Self {
        let id = id.to_string();
        let typ = T::NAME.to_string();
        let guid = format!("{}_{}", typ, id);
        let meta = RecordMeta {
            guid,
            typ,
            id,
            ..Default::default()
        };
        Self { meta, value }
    }

    /// Convert this record into an [UntypedRecord].
    ///
    /// This can be unwrapped by default as it only fails if the record value would not serialize
    /// to an object (which should be treated as a bug).
    pub fn into_untyped(self) -> Result<UntypedRecord, EncodingError>
    where
        T: Serialize,
    {
        let value = serde_json::to_value(self.value)?;
        let value = if let Value::Object(value) = value {
            value
        } else {
            return Err(EncodingError::NotAnObject);
        };
        let record = UntypedRecord {
            meta: self.meta,
            value,
        };
        Ok(record)
    }

    /// Convert this record into a JSON [Object].
    pub fn into_json_object(self) -> Result<JsonObject, EncodingError> {
        let value = serde_json::to_value(self)?;
        if let Value::Object(value) = value {
            Ok(value)
        } else {
            Err(EncodingError::NotAnObject)
        }
    }

    /// Validate the contained value (according to the [TypedValue] implementation)
    pub fn validate(&self) -> Result<(), ValidationError> {
        self.value.validate()
    }
}

impl<T> TypedRecord<T>
where
    T: Resolvable + Send,
{
    /// Resolve all references within this record into loaded records.
    pub async fn resolve_refs<R: Resolver + Send + Sync>(
        &mut self,
        resolver: &R,
    ) -> Result<(), MissingRefsError> {
        self.value.resolve_refs(resolver).await
    }

    /// Resolve all references, while consuming Self and returning it again with the resolved refs.
    pub async fn into_resolve_refs<R: Resolver + Send + Sync>(
        mut self,
        resolver: &R,
    ) -> Result<Self, MissingRefsError> {
        let _ = self.value.resolve_refs(resolver).await?;
        Ok(self)
    }

    /// Extract all loaded referenced records from within the record, converting the references
    /// back to IDs.
    pub fn extract_refs(&mut self) -> Vec<UntypedRecord> {
        self.value.extract_refs()
    }
}

impl<T> ElasticMapping for TypedRecord<T>
where
    T: ElasticMapping + Clone,
{
    fn elastic_mapping() -> serde_json::Value {
        let meta = RecordMeta::elastic_mapping();
        let meta = to_object(meta).unwrap();
        let inner = T::elastic_mapping();
        if !inner.is_object() {
            Value::Null
        } else {
            let mut object = to_object(inner).unwrap();
            object.insert(
                "$meta".to_string(),
                json!({
                    "type": "object",
                    "properties":  Value::Object(meta)
                }),
            );
            Value::Object(object)
        }
    }
}

fn to_object(value: Value) -> Option<JsonObject> {
    match value {
        Value::Object(object) => Some(object),
        _ => None,
    }
}
