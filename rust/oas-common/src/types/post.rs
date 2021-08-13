use crate::mapping::Mappable;
use crate::record::TypedValue;
use crate::reference::{self, Reference};
use crate::Resolvable;
use crate::Resolver;
use crate::UntypedRecord;
use crate::{ElasticMapping, MissingRefsError};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::Media;
// use super::Feed;

#[derive(Serialize, Deserialize, Debug, Clone, Default, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Post {
    pub identifier: Option<String>,
    pub headline: Option<String>,
    pub r#abstract: Option<String>,
    pub contributor: Option<String>,
    pub url: Option<String>,
    pub date_published: Option<String>,
    pub date_modified: Option<String>,
    pub description: Option<String>,
    pub in_language: Option<String>,
    pub licence: Option<String>,
    pub publisher: Option<String>,
    #[serde(default)]
    pub genre: Vec<String>,
    #[serde(default)]
    pub creator: Vec<String>,
    #[serde(default)]
    pub media: Vec<Reference<Media>>,
    // #[serde(default)]
    // pub feed: Reference<Feed>,
    pub transcript: Option<String>,

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
                "type": "nested",
                "include_in_parent": true,
                "properties": Media::elastic_mapping().unwrap_or(serde_json::Value::Object(serde_json::Map::new())),
            },
            "transcript": {
                "type": "text",
                "term_vector": "with_positions_payloads",
                "analyzer": "payload_delimiter"
            },
            "datePublished": {
                "type": "date"
            },
            "abstract":{
                "type":"text",
            },
            "contentUrl":{
                "type":"text",
                "fields":{
                    "keyword":{
                        "type":"keyword",
                        "ignore_above":256
                    }
                }
            },
            "contributor":{
                "properties":{
                    "name":{
                        "type":"text",
                        "fields":{
                            "keyword":{
                                "type":"keyword",
                                "ignore_above":256
                            }
                        }
                    }
                }
            },
            "creator":{
                "type":"text",
                "fields":{
                    "keyword":{
                        "type":"keyword",
                        "ignore_above":256
                    }
                }
            },
            "dateModified":{
                "type":"date"
            },
            "datePublished":{
                "type":"date"
            },
            "description":{
                "type":"text",
                "fields":{
                    "keyword":{
                        "type":"keyword",
                        "ignore_above":256
                    }
                }
            },
            "genre":{
                "type":"text",
                "fields":{
                    "keyword":{
                        "type":"keyword",
                        "ignore_above":256
                    }
                }
            },
            "headline":{
                "type":"text",
                "fields":{
                    "keyword":{
                        "type":"keyword",
                        "ignore_above":256
                    }
                }
            },
            "identifier":{
                "type":"text",
                "fields":{
                    "keyword":{
                        "type":"keyword",
                        "ignore_above":256
                    }
                }
            },
            "inLanguage":{
                "type":"text",
                "fields":{
                    "keyword":{
                        "type":"keyword",
                        "ignore_above":256
                    }
                }
            },
            "licence":{
                "type":"text",
                "fields":{
                    "keyword":{
                        "type":"keyword",
                        "ignore_above":256
                    }
                }
            },
            "publisher":{
                "type":"text",
                "fields":{
                    "keyword":{
                        "type":"keyword",
                        "ignore_above":256
                    }
                }
            },
            "url":{
                "type":"text",
                "fields":{
                    "keyword":{
                        "type":"keyword",
                        "ignore_above":256
                    }
                }
            }
        }))
    }
}
