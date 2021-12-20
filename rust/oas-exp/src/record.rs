use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use serde::ser::Error;
use serde::{Deserialize, Serialize};
use std::any::{Any, TypeId};
use std::marker::PhantomData;
use std::sync::Arc;
use thiserror::Error;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Blank;

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
pub struct RawRecord {
    id: Uuid,
    typ: String,
    value: serde_json::Value,
}

#[derive(JsonSchema, Serialize, Deserialize, Debug, Clone)]
pub struct Guid {
    id: Uuid,
    typ: String,
    rev: Option<String>,
}

// pub struct TypedRecord<T> {
//     record: Record,
//     phantom: PhantomData<T>,
// }

#[derive(Debug, Deserialize, JsonSchema)]
pub struct Record {
    #[serde(flatten)]
    raw: RawRecord,
    #[serde(default, skip)]
    typed: Option<Result<Box<dyn AsAny>, UpcastError>>,
    #[serde(default, skip)]
    raw_is_dirty: bool,
}

impl Serialize for Record {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if let Some(raw) = self.as_raw() {
            raw.serialize(serializer)
        } else {
            let raw = self
                .into_raw_cloned()
                .map_err(|err| S::Error::custom(err.to_string()))?;
            raw.serialize(serializer)
        }
    }
}

impl std::clone::Clone for Record {
    fn clone(&self) -> Self {
        Self {
            raw: self.raw.clone(),
            typed: self.typed.clone(),
            raw_is_dirty: self.raw_is_dirty,
        }
    }
}

#[derive(Error, Debug, Clone)]
pub enum UpcastError {
    #[error("Type mismatch: Wanted {0}, but got {1}")]
    TypeMismatch(String, String),
    #[error("Type not found: {0}")]
    TypeMissing(String),
    #[error("Decoding error: {0}")]
    Decoding(Arc<serde_json::Error>),
}

impl From<serde_json::Error> for UpcastError {
    fn from(e: serde_json::Error) -> Self {
        Self::Decoding(Arc::new(e))
    }
}

#[derive(Error, Debug)]
pub enum DowncastError {
    #[error("Failed upcast")]
    FailedUpcast(UpcastError),
    #[error("Encoding")]
    Encoding(#[from] serde_json::Error),
    #[error("Missing value")]
    MissingValue,
}

fn upcast<'s, T: RecordValue>(value: &'s Box<dyn AsAny>) -> Result<&'s T, UpcastError> {
    let value: &dyn std::any::Any = value.as_ref().as_any();
    let value: Option<&T> = <dyn std::any::Any>::downcast_ref::<T>(value);
    let value = value.ok_or(UpcastError::TypeMismatch(
        T::NAME.to_string(),
        "unknown".to_string(),
    ))?;
    Ok(value)
}

fn upcast_mut<'s, T: RecordValue>(value: &'s mut Box<dyn AsAny>) -> Result<&'s mut T, UpcastError> {
    let value: &mut dyn std::any::Any = value.as_mut().as_any_mut();
    let value: Option<&mut T> = <dyn std::any::Any>::downcast_mut::<T>(value);
    let value = value.ok_or(UpcastError::TypeMismatch(
        T::NAME.to_string(),
        "unknown".to_string(),
    ))?;
    Ok(value)
}

impl Record {
    pub fn new<T: RecordValue>(id: Uuid, value: T) -> Self {
        let raw = RawRecord {
            id,
            typ: T::NAME.to_string(),
            value: serde_json::Value::Null,
        };
        let record: Record = Record {
            raw,
            raw_is_dirty: true,
            typed: Some(Ok(Box::new(value))),
        };
        record
    }

    pub fn from_raw(raw: RawRecord) -> Self {
        Self {
            raw,
            raw_is_dirty: false,
            typed: None,
        }
    }

    pub fn from_json(json: serde_json::Value) -> serde_json::Result<Self> {
        let raw: RawRecord = serde_json::from_value(json)?;

        let record: Record = Record {
            raw,
            raw_is_dirty: false,
            typed: None,
        };
        Ok(record)
    }

    pub fn value<T: RecordValue>(&self) -> Result<&T, UpcastError> {
        match self.typed.as_ref() {
            None => Err(UpcastError::TypeMismatch(
                T::NAME.to_string(),
                "unknown".to_string(),
            )),
            Some(Err(err)) => Err(err.clone()),
            Some(Ok(typed)) => upcast(&typed),
        }
    }
    pub fn value_mut<T: RecordValue>(&mut self) -> Result<&mut T, UpcastError> {
        match self.typed.as_mut().unwrap() {
            Err(err) => Err(err.clone()),
            Ok(typed) => {
                let value = upcast_mut(typed)?;
                self.raw_is_dirty = true;
                Ok(value)
            }
        }
    }
    pub fn typ(&self) -> &str {
        &self.raw.typ
    }

    pub fn id(&self) -> &Uuid {
        &self.raw.id
    }

    pub fn into_upcast<T: RecordValue>(mut self) -> Result<Self, UpcastError> {
        self.upcast::<T>()?;
        Ok(self)
    }

    pub fn blank(self) -> Record {
        Record {
            raw: self.raw,
            typed: None,
            raw_is_dirty: false,
        }
    }

    pub fn upcast<T>(&mut self) -> Result<(), UpcastError>
    where
        T: RecordValue,
    {
        if self.typ() != T::NAME {
            return Err(UpcastError::TypeMismatch(
                T::NAME.to_string(),
                self.typ().to_string(),
            ));
        }
        if self.typed.is_none() {
            let value = deserialize_boxed::<T>(self.raw.value.clone());
            self.typed = Some(value);
        };
        match &self.typed {
            Some(Ok(typed)) => match typed.as_any().type_id() == TypeId::of::<T>() {
                true => Ok(()),
                // TODO: Make this !unreachable()?
                false => Err(UpcastError::TypeMismatch(
                    T::NAME.to_string(),
                    self.typ().to_string(),
                )),
            },
            Some(Err(e)) => Err(e.clone()),
            None => unreachable!(),
        }
    }

    pub fn into_raw(mut self) -> Result<RawRecord, DowncastError> {
        self.update_json()?;
        Ok(self.raw)
    }

    pub fn into_raw_cloned(&self) -> Result<RawRecord, DowncastError> {
        if self.raw_is_dirty {
            // TODO: Optimize by not cloning everything.
            let this = self.clone();
            this.into_raw()
        } else {
            Ok(self.raw.clone())
        }
    }

    pub fn as_raw(&self) -> Option<&RawRecord> {
        if !self.raw_is_dirty {
            Some(&self.raw)
        } else {
            None
        }
    }

    pub fn into_json_cloned(&self) -> Result<serde_json::Value, DowncastError> {
        Ok(serde_json::to_value(self.into_raw_cloned()?)?)
    }

    pub fn into_json(self) -> Result<serde_json::Value, DowncastError> {
        Ok(serde_json::to_value(self.into_raw()?)?)
    }

    pub fn value_as_json(&mut self) -> Result<&serde_json::Value, DowncastError> {
        self.update_json()?;
        Ok(&self.raw.value)
    }

    pub fn update_json(&mut self) -> Result<(), DowncastError> {
        if self.raw_is_dirty {
            match self.recreate_json_from_typed() {
                Ok(json) => {
                    self.raw_is_dirty = false;
                    self.raw.value = json;
                    Ok(())
                }
                Err(err) => Err(err),
            }
        } else {
            Ok(())
        }
    }

    fn recreate_json_from_typed(&self) -> Result<serde_json::Value, DowncastError> {
        let value = self.typed.as_ref().ok_or(DowncastError::MissingValue)?;
        match value {
            Ok(value) => {
                eprintln!("-- WORK serialize");
                let serialized = serde_json::to_value(value)?;
                Ok(serialized)
            }
            Err(err) => Err(DowncastError::FailedUpcast(err.clone())),
        }
    }
}
fn deserialize_boxed<T>(value: serde_json::Value) -> Result<Box<dyn AsAny>, UpcastError>
where
    T: RecordValue,
{
    eprintln!("-- WORK deserialize");
    let value = serde_json::from_value::<T>(value)?;
    let value: Box<dyn AsAny> = Box::new(value);
    Ok(value)
}

pub trait AsAny: erased_serde::Serialize + std::fmt::Debug + std::any::Any {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn clone_box(&self) -> Box<dyn AsAny>;
}

impl std::clone::Clone for Box<dyn AsAny> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

// impl<T> AsAny for T
// where
//     T: erased_serde::Serialize + std::fmt::Debug + std::clone::Clone + 'static,
// {
//     fn as_any(&self) -> &dyn Any {
//         self
//     }
//     fn as_any_mut(&mut self) -> &mut dyn Any {
//         self
//     }
//     fn clone_box(&self) -> Box<dyn AsAny> {
//         Box::new(self.clone())
//     }
// }

pub trait RecordValue: AsAny + DeserializeOwned {
    const NAME: &'static str;

    #[deprecated]
    fn typ() -> &'static str {
        Self::NAME
    }

    fn name() -> &'static str {
        Self::NAME
    }

    fn upcast(record: &mut Record) -> Result<(), UpcastError> {
        record.upcast::<Self>()
    }
}

erased_serde::serialize_trait_object!(AsAny);
