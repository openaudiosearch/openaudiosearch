use serde::{Deserialize, Serialize};

use crate::mapping::Mappable;
use crate::record::TypedValue;
use crate::reference::{self, Reference};
use crate::resolver::Resolver;
use crate::UntypedRecord;

use super::Media;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Post {
    pub headline: Option<String>,
    pub url: Option<String>,
    pub date_published: Option<String>,
    #[serde(default)]
    pub genre: Vec<String>,
    #[serde(flatten)]
    pub other: serde_json::Map<String, serde_json::Value>,
    pub media: Vec<Reference<Media>>,
}

impl TypedValue for Post {
    const NAME: &'static str = "oas.Post";
}

impl Mappable for Post {}

impl Post {
    pub async fn resolve_refs<R: Resolver + Send + Sync>(
        &mut self,
        resolver: &R,
    ) -> Result<(), R::Error> {
        resolver.resolve_refs(&mut self.media).await;
        Ok(())
    }

    pub fn extract_refs(&mut self) -> Vec<UntypedRecord> {
        reference::extract_refs(&mut self.media)
    }
}
