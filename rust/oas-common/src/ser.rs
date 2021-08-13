use chrono::prelude::*;
use serde::{de, Deserializer};
use std::fmt;
use std::str::FromStr;

pub fn deserialize_date<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
where
    D: Deserializer<'de>,
{
    match deserializer.deserialize_any(DateVisitor) {
        Ok(value) => Ok(Some(value)),
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
            let mut split: Vec<&str> = value.split(":").collect();
            split[..].reverse();

            let mut factor = 1.;
            let mut result = 0.;
            for part in split {
                let part: f32 = f32::from_str(&part).map_err(de::Error::custom)?;
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

    fn visit_str<E>(self, value: &str) -> Result<Vec<String>, E>
    where
        E: de::Error,
    {
        let mut result = Vec::new();
        result.push(value.into());
        Ok(result)
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
    type Value = DateTime<Utc>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a datetime string")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match DateTime::parse_from_rfc2822(value) {
            Ok(value) => Ok(value.with_timezone(&Utc)),
            Err(e) => Err(E::custom(format!("Parse error {} for {}", e, value))),
        }
    }
}
