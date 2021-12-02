use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{DecodingError, EncodingError, RecordMeta, TypedRecord, TypedValue};
use crate::JsonObject;

/// An untyped record is a record without static typing. The value is encoded as a JSON object.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
pub struct UntypedRecord {
    #[serde(rename = "$meta")]
    pub(crate) meta: RecordMeta,
    #[serde(flatten)]
    pub(crate) value: JsonObject,
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
    pub fn into_json_object(self) -> Result<JsonObject, EncodingError> {
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
        let res = json_patch::patch(&mut value, patch);
        match res {
            Ok(_) => match value {
                Value::Object(value) => {
                    self.value = value;
                    Ok(())
                }
                _ => Err(EncodingError::NotAnObject),
            },
            Err(err) => Err(err.into()),
        }
    }
}
