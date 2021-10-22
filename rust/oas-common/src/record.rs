use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::any::Any;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt;
use thiserror::Error;

use crate::task::TaskObject;
use crate::{ElasticMapping, MissingRefsError, Resolvable, Resolver};

pub type Object = serde_json::Map<String, serde_json::Value>;
pub type Record<T> = TypedRecord<T>;

/// An error that occurs while encoding a record.
#[derive(Error, Debug)]
pub enum EncodingError {
    #[error("Serialization failed")]
    SerdeJson(#[from] serde_json::Error),
    #[error("Serialization did not return an object")]
    NotAnObject,
    #[error("Invalid patch")]
    Patch(#[from] json_patch::PatchError),
}

/// An error that occurs while decoding a record.
#[derive(Error, Debug)]
pub enum DecodingError {
    #[error("Deserialization failed")]
    SerdeJson(#[from] serde_json::Error),
    #[error("Type mismatch: expected {0}, got {1}")]
    TypeMismatch(String, String),
    #[error("Deserialization did not return an object")]
    NotAnObject,
}

/// Record metadata.
#[derive(Serialize, Deserialize, Debug, Clone, Default, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct RecordMeta {
    guid: String,
    #[serde(rename = "type")]
    typ: String,
    id: String,

    #[serde(default)]
    jobs: JobMap, // TODO: Add more metadata?
                  // source: String,
                  // seq: u32,
                  // version: u32,
                  // timestamp: u32,
                  // tasks: HashMap<TaskName, TaskState>
}

impl RecordMeta {
    pub fn insert_job(&mut self, typ: &str, id: u64) {
        self.jobs
            .entry(typ.to_string())
            .or_insert_with(|| vec![])
            .push(id);
    }

    pub fn jobs(&self, typ: &str) -> Option<&Vec<u64>> {
        self.jobs.get(typ)
    }

    pub fn latest_job(&self, typ: &str) -> Option<u64> {
        self.jobs.get(typ).and_then(|vec| vec.last().map(|x| *x))
    }

    pub fn latest_jobs(&self) -> HashMap<String, u64> {
        self.jobs
            .iter()
            .filter_map(|(typ, jobs)| jobs.last().map(|id| (typ.to_string(), *id)))
            .collect()
    }
}

pub type JobMap = HashMap<String, Vec<u64>>;

impl ElasticMapping for RecordMeta {
    fn elastic_mapping() -> serde_json::Value {
        serde_json::json!({
            "guid": {
                "type": "keyword"
            },
            "id": {
                "type": "keyword"
            },
            "typ": {
                "type": "keyword"
            },
        })
    }
}

#[derive(Debug, Serialize)]
pub struct ValidationError {
    message: String,
}

impl<E> From<E> for ValidationError
where
    E: std::error::Error + Send + 'static,
{
    fn from(e: E) -> Self {
        Self::from_error(e)
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl ValidationError {
    pub fn with_message(message: String) -> Self {
        Self { message }
    }
    pub fn from_error<E>(error: E) -> Self
    where
        E: std::error::Error + Send + 'static,
    {
        Self {
            message: format!("{}", error),
        }
    }
}

/// A trait to implement on value structs for typed [Record]s.
pub trait TypedValue:
    fmt::Debug + Any + Serialize + DeserializeOwned + std::clone::Clone + Send
{
    /// A string to uniquely identify this record type.
    const NAME: &'static str;

    /// Get a human-readable label for this record.
    ///
    /// This method is optional and returns None by default. Record types may implement this method
    /// to return a title or headline.
    fn label(&self) -> Option<&'_ str> {
        None
    }

    /// Get the guid string for this record type and an id string.
    fn guid(id: &str) -> String {
        format!("{}_{}", Self::NAME, id)
    }

    fn validate(&self) -> Result<(), ValidationError> {
        Ok(())
    }
}

/// An untyped record is a record without static typing. The value is encoded as a JSON object.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
pub struct UntypedRecord {
    #[serde(rename = "$meta")]
    meta: RecordMeta,
    #[serde(flatten)]
    value: Object,
}

impl UntypedRecord {
    /// Create an untyped record from type and id strings and a [serde_json::Value].
    ///
    /// The value should be a [serde_json::Map], otherwise this method will return an
    /// [DecodingError::NotAnObject].
    pub fn with_typ_id_value(typ: &str, id: &str, value: Value) -> Result<Self, DecodingError> {
        let meta = RecordMeta {
            typ: typ.into(),
            id: id.into(),
            ..Default::default()
        };
        let value = match value {
            Value::Object(object) => object,
            _ => return Err(DecodingError::NotAnObject),
        };
        Ok(Self { meta, value })
    }

    pub fn into_typed_record<T: TypedValue + DeserializeOwned + Clone + 'static>(
        self,
    ) -> Result<TypedRecord<T>, DecodingError> {
        self.into_typed()
    }

    /// Convert this untyped record into a typed [Record].
    /// #[deprecated(note = "use into_typed")]
    pub fn into_typed<T: TypedValue + DeserializeOwned + Clone + 'static>(
        self,
    ) -> Result<TypedRecord<T>, DecodingError> {
        if self.meta.typ.as_str() != T::NAME {
            return Err(DecodingError::TypeMismatch(
                T::NAME.to_string(),
                self.meta.typ,
            ));
        }
        let value: T = serde_json::from_value(Value::Object(self.value))?;
        let record = TypedRecord {
            meta: self.meta,
            value,
        };
        Ok(record)
    }

    /// Convert the untyped record into a JSON [Object].
    pub fn into_json_object(self) -> Result<Object, EncodingError> {
        let value = serde_json::to_value(self)?;
        if let Value::Object(value) = value {
            Ok(value)
        } else {
            Err(EncodingError::NotAnObject)
        }
    }

    /// Get the guid of the record.
    pub fn guid(&self) -> &str {
        &self.meta.guid
    }

    /// Get the id of the record.
    pub fn id(&self) -> &str {
        &self.meta.id
    }

    /// Get the type of the record.
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

    /// Merge this record's value with another JSON value.
    pub fn merge_json_value(
        &mut self,
        value_to_merge: serde_json::Value,
    ) -> Result<(), EncodingError> {
        // TODO: Get rid of this clone?
        let mut value = Value::Object(self.value.clone());
        json_patch::merge(&mut value, &value_to_merge);
        // TODO: Validate the result?
        match value {
            Value::Object(value) => {
                self.value = value;
                Ok(())
            }
            _ => Err(EncodingError::NotAnObject),
        }
    }

    pub fn apply_json_patch(&mut self, patch: &json_patch::Patch) -> Result<(), EncodingError> {
        let mut value = Value::Object(self.value.clone());
        let res = json_patch::patch(&mut value, &patch);
        match res {
            Ok(_) => match value {
                Value::Object(value) => {
                    self.value = value;
                    Ok(())
                }
                _ => Err(EncodingError::NotAnObject.into()),
            },
            Err(err) => Err(err.into()),
        }
    }
}

impl<T> TryFrom<UntypedRecord> for TypedRecord<T>
where
    T: TypedValue,
{
    type Error = DecodingError;
    fn try_from(record: UntypedRecord) -> Result<Self, Self::Error> {
        record.into_typed_record()
    }
}

impl<T> TryFrom<TypedRecord<T>> for UntypedRecord
where
    T: TypedValue,
{
    type Error = EncodingError;
    fn try_from(record: TypedRecord<T>) -> Result<Self, Self::Error> {
        record.into_untyped_record()
    }
}

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
    T: TaskObject,
{
    pub fn task_states(&self) -> Option<&<T as TaskObject>::TaskStates> {
        self.value.task_states()
    }

    pub fn task_states_mut(&mut self) -> Option<&mut <T as TaskObject>::TaskStates> {
        self.value.task_states_mut()
    }
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
    pub fn into_untyped_record(self) -> Result<UntypedRecord, EncodingError>
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
    pub fn into_json_object(self) -> Result<Object, EncodingError> {
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

fn to_object(value: Value) -> Option<Object> {
    match value {
        Value::Object(object) => Some(object),
        _ => None,
    }
}

// impl<T> TypedRecord<T>
// where
//     T: TypedValue,
// {
//     fn downcast<T: 'static>(&self) -> Option<&T> {
//         let value: &dyn Any = &self.value;
//         if let Some(value) = value.downcast_ref::<T>() {
//             Some(value)
//         } else {
//             None
//         }
//     }

//     fn downcast_mut<T: 'static>(&mut self) -> Option<&mut T> {
//         let value: &mut dyn Any = &mut self.value;
//         if let Some(mut value) = value.downcast_mut::<T>() {
//             Some(value)
//         } else {
//             None
//         }
//     }

//     fn into_downcast<T>(self) -> Option<T> {
//         // let value: Box<dyn Any> = self.value.downcast();
//         downcast_box::<T>(self.value)
//         // let value = *self.value;
//         // let value: dyn Any = self.value;
//         // let value: Box<dyn Any> = self.value;
//         // if let Ok(value) = Box<Any>::downcast::<T>(self.value) {
//         //     Some(*value)
//         // } else {
//         //     None
//         // }
//         // None
//     }

//     fn downcast_box<T>(value: Box<dyn Any>) -> Option<T>
//     where
//         T: 'static,
//     {
//         if let Ok(value) = value.downcast::<T>() {
//             Some(*value)
//         } else {
//             None
//         }
//     }
// }
