use crate::Object;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MappingError {
    #[error("(De)serialization error")]
    Json(#[from] serde_json::Error),
    #[error("Expected an array")]
    NotAnArray,
    #[error("Expected an object")]
    NotAnObject,
}

pub trait Mappable: Sized + DeserializeOwned {
    fn from_fieldmap<T>(value: T, fieldmap: &FieldMap) -> Result<Self, MappingError>
    where
        T: Serialize,
    {
        let value = serde_json::to_value(value)?;
        Self::from_value(value, fieldmap)
    }

    fn from_json(json: impl AsRef<str>, fieldmap: &FieldMap) -> Result<Self, MappingError> {
        let value: Value = serde_json::from_str(json.as_ref())?;
        Self::from_value(value, fieldmap)
    }

    fn from_value(value: Value, fieldmap: &FieldMap) -> Result<Self, MappingError> {
        let target = fieldmap.apply_json_value(value)?;
        let result: Self = serde_json::from_value(target)?;
        Ok(result)
    }

    fn from_object(object: Object, fieldmap: &FieldMap) -> Result<Self, MappingError> {
        let target = fieldmap.apply_json_object(object)?;
        let result: Self = serde_json::from_value(serde_json::Value::Object(target))?;
        Ok(result)
    }
}

impl Mappable for Value {}

// pub type FieldMap = HashMap<String, FieldMapping>;

#[derive(Default, Serialize, Debug, Deserialize)]
pub struct FieldMap {
    #[serde(flatten)]
    pub(crate) inner: HashMap<String, FieldMapping>,
}

impl FieldMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, key: String, field_mapping: FieldMapping) {
        self.inner.insert(key, field_mapping);
    }

    pub fn inner(&self) -> &HashMap<String, FieldMapping> {
        &self.inner
    }

    pub fn get(&self, key: &str) -> Option<&FieldMapping> {
        self.inner.get(key)
    }

    pub fn apply_json_value(&self, source: Value) -> Result<Value, MappingError> {
        match source {
            Value::Object(object) => self.apply_json_object(object).map(Value::Object),
            _ => Err(MappingError::NotAnObject),
        }
        // if let Value::Object(source) = source {
        //     Ok(Value::Object(self.apply_json_object(source)?))
        // } else {
        //     Err(MappingError::NotAnObject)
        // }
    }

    pub fn apply_json_object(&self, source: Object) -> Result<Object, MappingError> {
        let mut target = Object::new();
        for (key, value) in source.into_iter() {
            if let Some(field_mapping) = self.get(&key) {
                let target_value = field_mapping.apply(value)?;
                let target_key = field_mapping.target_key().to_string();
                target.insert(target_key, target_value);
            }
        }
        Ok(target)
    }

    pub fn reverse(self) -> Self {
        let mut target_map = FieldMap::new();
        for (source_key, field_mapping) in self.inner.into_iter() {
            let (target_key, reveresed_field_mapping) = field_mapping.reverse(source_key);
            target_map.insert(target_key, reveresed_field_mapping);
        }
        target_map
    }
}

// pub struct FieldMap {
//     source_type: String,
//     target_type: String
//     field_map: HashMap<String, FieldMapping>
// }

// pub fn reverse_field_map(field_map: FieldMap) -> FieldMap {
//     let mut target_map = FieldMap::new();
//     for (source_key, field_mapping) in field_map.into_iter() {
//         let (target_key, reveresed_field_mapping) = field_mapping.reverse(source_key);
//         target_map.insert(target_key, reveresed_field_mapping);
//     }
//     target_map
// }

#[derive(Default, Serialize, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FieldMapping {
    target_key: String,
    #[serde(default)]
    into_array: bool,
    #[serde(default)]
    into_single: bool,
    // regex_match: Option<String>,
    // json_path_match: Option<String>,
}

impl FieldMapping {
    pub fn target_key(&self) -> &str {
        &self.target_key
    }

    pub fn apply(&self, value: Value) -> Result<Value, MappingError> {
        if self.into_array {
            Ok(Value::Array(vec![value]))
        } else if self.into_single {
            match value {
                Value::Array(list) if !list.is_empty() => Ok(list.into_iter().next().unwrap()),
                Value::Array(list) if list.is_empty() => Ok(Value::Null),
                _ => Err(MappingError::NotAnArray),
            }
        } else {
            Ok(value)
        }
    }

    pub fn reverse(self, source_key: String) -> (String, FieldMapping) {
        let target_key = self.target_key;
        let reversed_mapping = FieldMapping {
            target_key: source_key,
            into_array: self.into_single,
            into_single: self.into_array,
        };
        (target_key, reversed_mapping)
    }
}

// pub struct FieldMappingBuilder {
//     target_key: String,
//     into_array: bool,
//     into_single: bool,
//     regex_match: Option<String>,
//     json_path_match: Option<String>,
// }

// impl FiledMappingBuilder {
//     fn new(target_key: String) -> Self {
//         Self {
//             target_key,
//             Default::default()
//         }
//     }
//     fn into_single(self, into_single: bool) -> self {
//         self.into_single = into_single;
//         self
//     }

//     fn build(self) -> FieldMapping {
//         FieldMapping {
//             ..
//         }
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    #[test]
    fn it_works() {
        let source = r#"
        {
            "title": "hello world",
            "tags": ["foo", "bar"]
        }
        "#;

        let expected = r#"
        {
            "headline": "hello world",
            "primaryTag": "foo"
        }
        "#;

        let field_map = r#"
        {
            "title": {
                "targetKey": "headline"
            },
            "tags": {
                "targetKey": "primaryTag",
                "intoSingle": true
            }
        }
        "#;

        let expected = reserialize(expected);
        let source = reserialize(source);

        let field_map: FieldMap =
            serde_json::from_str(field_map).expect("failed to parse field map");

        let target: Value = Value::from_json(&source, &field_map).unwrap();
        let target = serde_json::to_string(&target).unwrap();

        assert!(target == expected);

        let reversed_map = field_map.reverse();
        eprintln!("reversed map: {:?}", reversed_map);
        let reversed_target: Value = Value::from_json(&expected, &reversed_map).unwrap();
        let reversed_target = serde_json::to_string(&reversed_target).unwrap();

        let expected = r#"
        {
            "title": "hello world",
            "tags": ["foo"]
        }
        "#;
        let expected = reserialize(expected);

        println!("source:\n{}", source);
        println!("reversed_target:\n{}", reversed_target);
        assert!(reversed_target == expected);
    }

    fn reserialize(input: &str) -> String {
        let output: Value = serde_json::from_str(input).expect("failed to parse");
        let output = serde_json::to_string(&output).expect("failed to serialize");
        output
    }
}
