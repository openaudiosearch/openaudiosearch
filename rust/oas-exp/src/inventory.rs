use crate::{Record, RecordValue, Schema, UpcastError};
use std::collections::HashMap;

pub struct Inventory {
    schemas: HashMap<String, Schema>,
}
impl Inventory {
    pub fn new() -> Self {
        Self {
            schemas: HashMap::new(),
        }
    }

    pub fn insert(&mut self, typ: Schema) {
        self.schemas.insert(typ.name().to_string(), typ);
    }

    pub fn for_typ<T: RecordValue>(&self) -> Option<&Schema> {
        self.get(T::NAME)
    }

    pub fn get(&self, typ: &str) -> Option<&Schema> {
        self.schemas.get(typ)
    }

    pub fn upcast(&self, record: &mut Record) -> Result<(), UpcastError> {
        let typ = self.schemas.get(record.typ());
        if let Some(typ) = typ {
            typ.upcast(record)?;
            Ok(())
        } else {
            Err(UpcastError::TypeMissing(record.typ().to_string()))
        }
    }
}
