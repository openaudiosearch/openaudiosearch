use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::any::Any;
use std::convert::TryFrom;
use std::fmt;
use thiserror::Error;

pub type Object = serde_json::Map<String, serde_json::Value>;
pub type Record<T> = TypedRecord<T>;

#[derive(Error, Debug)]
pub enum EncodingError {
    #[error("Serialization failed")]
    SerdeJson(#[from] serde_json::Error),
    #[error("Serialization did not return an object")]
    NotAnObject,
}

#[derive(Error, Debug)]
pub enum DecodingError {
    #[error("Deserialization failed")]
    SerdeJson(#[from] serde_json::Error),
    #[error("Type mismatch: expected {0}, got {1}")]
    TypeMismatch(String, String),
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct RecordMeta {
    guid: String,
    #[serde(rename = "type")]
    typ: String,
    id: String,
    source: String,
    seq: u32,
    version: u32,
    timestamp: u32,
}

pub trait TypedValue: fmt::Debug + Any + Serialize + DeserializeOwned + std::clone::Clone {
    const NAME: &'static str;
    // fn get_name(self) -> &str;
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UntypedRecord {
    #[serde(flatten)]
    meta: RecordMeta,
    value: Object,
}

impl UntypedRecord {
    pub fn into_typed_record<T: TypedValue + DeserializeOwned + Clone + 'static>(
        self,
    ) -> Result<TypedRecord<T>, DecodingError> {
        if self.meta.typ.as_str() != T::NAME {
            return Err(DecodingError::TypeMismatch(
                T::NAME.to_string(),
                self.meta.typ.clone(),
            ));
        }
        let value: T = serde_json::from_value(Value::Object(self.value))?;
        let record = TypedRecord {
            meta: self.meta,
            value,
        };
        Ok(record)
    }

    pub fn guid(&self) -> &str {
        &self.meta.guid
    }

    pub fn id(&self) -> &str {
        &self.meta.id
    }

    pub fn typ(&self) -> &str {
        &self.meta.typ
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TypedRecord<T>
where
    T: Clone,
{
    #[serde(flatten)]
    pub meta: RecordMeta,
    pub value: T,
}

impl<T> TypedRecord<T>
where
    T: TypedValue,
{
    pub fn guid(&self) -> &str {
        &self.meta.guid
    }

    pub fn id(&self) -> &str {
        &self.meta.id
    }

    pub fn typ(&self) -> &str {
        &self.meta.typ
    }

    pub fn from_id_and_value(id: String, value: T) -> Self {
        let typ = T::NAME.to_string();
        let guid = format!("{}_{}", typ, id);
        let meta = RecordMeta {
            guid,
            id,
            typ,
            ..Default::default()
        };
        Self { meta, value }
    }

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

    pub fn into_json_object(self) -> Result<Object, EncodingError> {
        let value = serde_json::to_value(self)?;
        if let Value::Object(value) = value {
            Ok(value)
        } else {
            Err(EncodingError::NotAnObject)
        }
    }
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

    // fn into_downcast<T>(self) -> Option<T> {
    //     // let value: Box<dyn Any> = self.value.downcast();
    //     downcast_box::<T>(self.value)
    //     // let value = *self.value;
    //     // let value: dyn Any = self.value;
    //     // let value: Box<dyn Any> = self.value;
    //     // if let Ok(value) = Box<Any>::downcast::<T>(self.value) {
    //     //     Some(*value)
    //     // } else {
    //     //     None
    //     // }
    //     // None
    // }

    // fn downcast_box<T>(value: Box<dyn Any>) -> Option<T>
    // where
    //     T: 'static,
    // {
    //     if let Ok(value) = value.downcast::<T>() {
    //         Some(*value)
    //     } else {
    //         None
    //     }
}

// #[derive(Serialize, Deserialize, Debug)]
// pub enum Value {
//     Untyped(JsValue),
//     Typed(Box<dyn TypedValue>),
// }

// #[derive(Serialize, Deserialize, Debug, Clone)]
// pub struct AudioObject {
//     duration: u32,
//     encoding_format: String,
// }
// impl TypedValue for AudioObject {
//     const NAME: &'static str = "oas.AudioObject";
//     // fn get_name(&self) -> &str {
//     //     "oas.Audio"
//     // }
// }

// #[derive(Serialize, Deserialize, Debug, Clone)]
// pub struct FileObject {
//     content_url: String,
// }
// impl TypedValue for FileObject {
//     const NAME: &'static str = "oas.FileObject";
//     // fn get_name(&self) -> &str {
//     //     "oas.File"
//     // }
// }
