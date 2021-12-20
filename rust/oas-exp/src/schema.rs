use crate::{Record, RecordValue, UpcastError};
use schemars::{gen::SchemaGenerator, schema::Schema as SchemarsSchema, JsonSchema};

pub struct Schema {
    name: String,
    upcast: Box<dyn 'static + Fn(&mut Record) -> Result<(), UpcastError>>,
    schema: Box<dyn 'static + Fn(&mut SchemaGenerator) -> SchemarsSchema>,
}

impl Schema {
    fn new<F1, F2>(name: impl ToString, upcast: F1, schema: F2) -> Self
    where
        F1: 'static + Fn(&mut Record) -> Result<(), UpcastError>,
        F2: 'static + Fn(&mut SchemaGenerator) -> SchemarsSchema,
    {
        Self {
            name: name.to_string(),
            upcast: Box::new(upcast),
            schema: Box::new(schema),
        }
    }

    pub fn from_value_typ<T: RecordValue + JsonSchema>() -> Self {
        Self::new(T::NAME, T::upcast, T::json_schema)
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn upcast(&self, record: &mut Record) -> Result<(), UpcastError> {
        (self.upcast)(record)
    }

    pub fn schema(&self, gen: &mut SchemaGenerator) -> SchemarsSchema {
        (self.schema)(gen)
    }
}
