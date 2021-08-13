use crate::mapping::Mappable;
use crate::record::TypedValue;
use crate::ElasticMapping;
use schemars::JsonSchema;
use serde::{de, Deserialize, Deserializer, Serialize};
use serde_json::json;
use std::fmt;
use std::str::FromStr;

#[derive(Serialize, Deserialize, Debug, Clone, Default, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Media {
    pub content_url: String,
    pub encoding_format: Option<String>,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_duration")]
    pub duration: Option<f32>,
    pub transcript: Option<Transcript>,
    pub nlp: Option<serde_json::Value>,

    #[serde(flatten)]
    pub other: serde_json::Map<String, serde_json::Value>,
}

fn deserialize_duration<'de, D>(deserializer: D) -> Result<Option<f32>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = deserializer.deserialize_any(F32Visitor);
    match value {
        Ok(value) if value == 0. => Ok(None),
        Ok(value) => Ok(Some(value)),
        Err(err) => Err(err),
    }
}
struct F32Visitor;
impl<'de> de::Visitor<'de> for F32Visitor {
    type Value = f32;
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string representation of a duration or a duration as a float number")
    }

    fn visit_f64<E>(self, value: f64) -> Result<f32, E>
    where
        E: de::Error,
    {
        Ok(value as f32)
    }

    fn visit_u64<E>(self, value: u64) -> Result<f32, E>
    where
        E: de::Error,
    {
        Ok(value as f32)
    }

    fn visit_i64<E>(self, value: i64) -> Result<f32, E>
    where
        E: de::Error,
    {
        Ok(value as f32)
    }

    fn visit_str<E>(self, value: &str) -> Result<f32, E>
    where
        E: de::Error,
    {
        if value.is_empty() {
            return Ok(0.);
        }
        let float = value.parse::<f32>().map_err(|_err| {
            E::invalid_value(
                de::Unexpected::Str(value),
                &"a string representation of a f64",
            )
        });
        if let Ok(float) = float {
            Ok(float)
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
            return Ok(result);
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, JsonSchema)]
pub struct Transcript {
    pub text: String,
    pub parts: Vec<TranscriptPart>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, JsonSchema)]
pub struct TranscriptPart {
    pub conf: f32,
    pub start: f32,
    pub end: f32,
    pub word: String,
}

impl TypedValue for Media {
    const NAME: &'static str = "oas.Media";
}

impl Mappable for Media {}

impl ElasticMapping for Media {
    fn elastic_mapping() -> Option<serde_json::Value> {
        Some(json!({
            "contentUrl":{
                "type":"text",
            },
            "duration": {
                "type": "float"
            },
            "encodingFormat": {
                "type": "keyword"
            },
            "nlp": {
                "type": "object"
            },
            "transcript": {
                "type": "object"
            }
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn deserialize_duration() {
        let source = r#"
        {
            "contentUrl": "foo",
            "duration": "02:03"
        }
        "#;

        let media: Media = serde_json::from_str(source).expect("failed to deserialize");
        assert_eq!(media.duration, Some(123.));

        let source = r#"
       {
           "contentUrl": "foo",
           "duration": "02:03:01"
       }
       "#;

        let media: Media = serde_json::from_str(source).expect("failed to deserialize");
        assert_eq!(media.duration, Some(7381.));

        let source = r#"
       {
           "contentUrl": "foo",
           "duration": "64"
       }
       "#;

        let media: Media = serde_json::from_str(source).expect("failed to deserialize");
        assert_eq!(media.duration, Some(64.));

        let source = r#"
        {
            "contentUrl": "foo",
            "duration": 123
        }
        "#;

        let media: Media = serde_json::from_str(source).expect("failed to deserialize");
        assert_eq!(media.duration, Some(123.));

        let source = r#"
        {
            "contentUrl": "foo",
            "duration": 562.5011
        }
        "#;

        let media: Media = serde_json::from_str(source).expect("failed to deserialize");
        assert_eq!(media.duration, Some(562.5011));

        let source = r#"
       {
           "contentUrl": "foo"
       }
       "#;

        let media: Media = serde_json::from_str(source).expect("failed to deserialize");
        assert_eq!(media.duration, None);
    }
}
