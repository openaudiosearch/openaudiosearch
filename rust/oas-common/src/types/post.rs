use super::{Feed, Media};
use crate::mapping::Mappable;
use crate::record::TypedValue;
use crate::reference::{self, Reference};
use crate::ser;
use crate::task::{TaskObject, TaskState};
use crate::{ElasticMapping, MissingRefsError};
use crate::{Record, Resolvable, Resolver, UntypedRecord};
use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize, Deserialize, Debug, Clone, Default, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Post {
    pub identifier: Option<String>,
    pub headline: Option<String>,
    pub r#abstract: Option<String>,
    pub description: Option<String>,
    pub in_language: Option<String>,
    pub licence: Option<String>,
    pub url: Option<String>,

    #[serde(default, deserialize_with = "ser::deserialize_date")]
    pub date_published: Option<DateTime<Utc>>,

    #[serde(default, deserialize_with = "ser::deserialize_date")]
    pub date_modified: Option<DateTime<Utc>>,

    #[serde(default, deserialize_with = "ser::deserialize_multiple")]
    pub contributor: Vec<String>,

    pub publisher: Option<String>,

    #[serde(default, deserialize_with = "ser::deserialize_multiple")]
    pub genre: Vec<String>,

    #[serde(default, deserialize_with = "ser::deserialize_multiple")]
    pub creator: Vec<String>,

    #[serde(default)]
    pub media: Vec<Reference<Media>>,

    #[serde(default)]
    pub feeds: Vec<Reference<Feed>>,

    pub transcript: Option<String>,

    pub nlp: Option<serde_json::Value>,

    #[serde(flatten)]
    pub other: serde_json::Map<String, serde_json::Value>,
    pub tasks: PostTasks,
}

impl TypedValue for Post {
    const NAME: &'static str = "oas.Post";
}

impl Mappable for Post {}

#[derive(Serialize, Deserialize, Debug, Clone, Default, JsonSchema)]
pub struct PostTasks {
    #[serde(deserialize_with = "ser::deserialize_null_default")]
    pub nlp: TaskState,
}

impl TaskObject for Post {
    type TaskStates = PostTasks;
    fn task_states(&self) -> Option<&Self::TaskStates> {
        Some(&self.tasks)
    }
    fn task_states_mut(&mut self) -> Option<&mut Self::TaskStates> {
        Some(&mut self.tasks)
    }
}

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
    fn elastic_mapping() -> serde_json::Value {
        json!({
            "tasks": {
                "type": "object",
                "enabled": false
            },
            "media": {
                "type": "nested",
                "include_in_parent": true,
                "properties": Record::<Media>::elastic_mapping()
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
                "type":"keyword",
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
            },
            "feeds": {
                "type":"keyword",
            }
        })
    }
}
