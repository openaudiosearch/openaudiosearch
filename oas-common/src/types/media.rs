use crate::mapping::Mappable;
use crate::record::TypedValue;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Media {
    pub content_url: String,
    pub encoding_format: Option<String>,
    pub transcript: Option<String>,
    pub duration: Option<f32>,

    #[serde(flatten)]
    pub other: serde_json::Map<String, serde_json::Value>,
}

impl TypedValue for Media {
    const NAME: &'static str = "oas.Media";
}

impl Mappable for Media {}
