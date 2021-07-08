use serde::{Deserialize, Serialize};

use crate::mapping::Mappable;
use crate::record::TypedValue;
use crate::reference::{self, Reference};
use crate::Resolvable;

use crate::Resolver;
use crate::UntypedRecord;
use crate::{ElasticMapping, MissingRefsError};
use serde_json::json;

use super::Media;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Post {
    pub identifier: Option<String>,
    pub headline: Option<String>,
    pub url: Option<String>,
    pub date_published: Option<String>,
    #[serde(default)]
    pub genre: Vec<String>,
    pub media: Vec<Reference<Media>>,

    #[serde(flatten)]
    pub other: serde_json::Map<String, serde_json::Value>,
}

impl TypedValue for Post {
    const NAME: &'static str = "oas.Post";
}

impl Mappable for Post {}

#[async_trait::async_trait]
impl Resolvable for Post {
    async fn resolve_refs<R: Resolver + Send + Sync>(
        &mut self,
        resolver: &R,
    ) -> Result<(), MissingRefsError> {
        resolver.resolve_refs(&mut self.media).await
    }

    fn extract_refs(&mut self) -> Vec<UntypedRecord> {
        reference::extract_refs(&mut self.media)
    }
}

impl ElasticMapping for Post {
    fn elastic_mapping() -> Option<serde_json::Value> {
        Some(json!({
            "media": {
                "type": "nested"
            },
            "datePublished": {
                "type": "date"
            }
        }))
    }
}
