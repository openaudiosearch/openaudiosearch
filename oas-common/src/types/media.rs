use crate::mapping::Mappable;
use crate::record::TypedValue;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Media {
    pub identifier: Option<String>,
    pub headline: Option<String>,
    pub url: Option<String>,
    pub content_url: Option<String>,
    pub encoding_format: Option<String>,
    pub transcript: Option<String>,
    pub duration: Option<f32>,
    pub date_published: Option<String>,
    #[serde(default)]
    pub genre: Vec<String>,
    #[serde(flatten)]
    pub other: serde_json::Map<String, serde_json::Value>,
}

impl TypedValue for Media {
    const NAME: &'static str = "oas.Media";
}

impl Mappable for Media {}
