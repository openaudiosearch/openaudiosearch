use crate::mapping::Mappable;
use crate::record::TypedValue;
use crate::ElasticMapping;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize, Deserialize, Debug, Clone, Default, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Media {
    pub content_url: String,
    pub encoding_format: Option<String>,
    pub duration: Option<f32>,
    pub transcript: Option<Transcript>,
    pub nlp: Option<serde_json::Value>,

    #[serde(flatten)]
    pub other: serde_json::Map<String, serde_json::Value>,
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
