use crate::mapping::Mappable;
use crate::record::TypedValue;
use crate::ElasticMapping;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Media {
    pub content_url: String,
    pub encoding_format: Option<String>,
    pub duration: Option<f32>,
    pub transcript: Option<serde_json::Value>,
    pub nlp: Option<serde_json::Value>,

    #[serde(flatten)]
    pub other: serde_json::Map<String, serde_json::Value>,
}

impl TypedValue for Media {
    const NAME: &'static str = "oas.Media";
}

impl Mappable for Media {}

impl ElasticMapping for Media {
    fn elastic_mapping() -> Option<serde_json::Value> {
        None
    }
}
