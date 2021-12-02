use chrono::prelude::*;
use serde::{de, de::Deserialize, Deserializer};
use std::fmt;
use std::marker::PhantomData;
use std::str::FromStr;

pub type JsonObject = serde_json::Map<String, serde_json::Value>;

pub fn deserialize_null_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    T: Default + Deserialize<'de>,
    D: Deserializer<'de>,
{
    let opt = Option::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}

pub fn deserialize_date<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
where
    D: Deserializer<'de>,
{
    match deserializer.deserialize_any(DateVisitor) {
        Ok(value) => Ok(value),
        Err(e) => Err(e),
    }
}

pub fn deserialize_multiple<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_any(VecVisitor)
}

pub fn deserialize_duration<'de, D>(deserializer: D) -> Result<Option<f32>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = deserializer.deserialize_any(F32Visitor);
    match value {
        Ok(Some(value)) => Ok(Some(value)),
        Ok(None) => Ok(None),
        Err(err) => Err(err),
    }
}
struct F32Visitor;
impl<'de> de::Visitor<'de> for F32Visitor {
    type Value = Option<f32>;
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string representation of a duration or a duration as a float number")
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(None)
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(None)
    }

    fn visit_f64<E>(self, value: f64) -> Result<Option<f32>, E>
    where
        E: de::Error,
    {
        Ok(Some(value as f32))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Option<f32>, E>
    where
        E: de::Error,
    {
        Ok(Some(value as f32))
    }

    fn visit_i64<E>(self, value: i64) -> Result<Option<f32>, E>
    where
        E: de::Error,
    {
        Ok(Some(value as f32))
    }

    fn visit_str<E>(self, value: &str) -> Result<Option<f32>, E>
    where
        E: de::Error,
    {
        if value.is_empty() {
            return Ok(None);
        }
        let float = value.parse::<f32>().map_err(|_err| {
            E::invalid_value(
                de::Unexpected::Str(value),
                &"a string representation of a f64",
            )
        });
        if let Ok(float) = float {
            Ok(Some(float))
        } else {
            let mut split: Vec<&str> = value.split(':').collect();
            split[..].reverse();

            let mut factor = 1.;
            let mut result = 0.;
            for part in split {
                let part: f32 = f32::from_str(part).map_err(de::Error::custom)?;
                result += part * factor;
                factor *= 60.;
            }
            Ok(Some(result))
        }
    }
}

struct VecVisitor;
impl<'de> de::Visitor<'de> for VecVisitor {
    type Value = Vec<String>;
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string with comma separated values or a single value")
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(vec![])
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(vec![])
    }

    fn visit_str<E>(self, value: &str) -> Result<Vec<String>, E>
    where
        E: de::Error,
    {
        Ok(vec![value.to_string()])
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: de::SeqAccess<'de>,
    {
        let mut result = vec![];
        while let Some(s) = seq.next_element::<String>()? {
            result.push(s);
        }
        Ok(result)
    }
}

struct DateVisitor;
impl<'de> de::Visitor<'de> for DateVisitor {
    type Value = Option<DateTime<Utc>>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a datetime string")
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(None)
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(None)
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match DateTime::parse_from_rfc2822(value) {
            Ok(value) => Ok(Some(value.with_timezone(&Utc))),
            Err(_e) => match DateTime::parse_from_rfc3339(value) {
                Ok(value) => Ok(Some(value.with_timezone(&Utc))),
                Err(_e) => match diligent_date_parser::parse_date(value) {
                    Some(value) => Ok(Some(value.with_timezone(&Utc))),
                    None => Err(E::custom(format!(
                        "Parse error for {}: Invalid date",
                        value
                    ))),
                },
            },
        }
    }
}

pub fn deserialize_multiple_objects<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    struct MultiVisitor<T>(PhantomData<T>);

    impl<'de, T: Deserialize<'de>> de::Visitor<'de> for MultiVisitor<T> {
        type Value = Vec<T>;
        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("an array of objects or a single object")
        }

        fn visit_unit<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(vec![])
        }

        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(vec![])
        }

        fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
        where
            A: de::MapAccess<'de>,
        {
            match Deserialize::deserialize(de::value::MapAccessDeserializer::new(map)) {
                Ok(value) => Ok(vec![value]),
                Err(err) => Err(err),
            }
        }

        fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
        where
            A: de::SeqAccess<'de>,
        {
            Deserialize::deserialize(de::value::SeqAccessDeserializer::new(seq))
        }
    }

    deserializer.deserialize_any(MultiVisitor::<T>(PhantomData))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    #[test]
    fn deserialize_one_or_vec() {
        #[derive(Deserialize, Debug)]
        struct Element {
            title: String,
            count: usize,
        }

        #[derive(Deserialize, Debug)]
        struct Root {
            cardinality: String,
            #[serde(deserialize_with = "deserialize_multiple_objects")]
            els: Vec<Element>,
        }

        let src1 = r#"
        {
            "cardinality": "many",
            "els": [
                { "title": "first", "count": 1 },
                { "title": "second", "count": 2 }
            ]
        }
        "#;

        let src2 = r#"
        {
            "cardinality": "single",
            "els": { "title": "first", "count": 1 }
        }
        "#;

        let res1: Root = serde_json::from_str(src1).expect("failed to deserialize many");
        let res2: Root = serde_json::from_str(src2).expect("failed to deserialize single");
        eprintln!("res1 {:#?}", res1);
        eprintln!("res2 {:#?}", res2);
    }
}
