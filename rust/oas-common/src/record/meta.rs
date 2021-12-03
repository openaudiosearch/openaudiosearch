use crate::{ElasticMapping, JobsLog};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Record metadata.
#[derive(Serialize, Deserialize, Debug, Clone, Default, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct RecordMeta {
    // TODO: Add more metadata?
    // source: String,
    // seq: u32,
    // version: u32,
    // timestamp: u32,
    pub(crate) guid: String,
    #[serde(rename = "type")]
    pub(crate) typ: String,
    pub(crate) id: String,
    #[serde(default)]
    pub(crate) jobs: JobsLog,
}

impl RecordMeta {
    pub fn jobs(&self) -> &JobsLog {
        &self.jobs
    }

    pub fn jobs_mut(&mut self) -> &mut JobsLog {
        &mut self.jobs
    }
}

impl ElasticMapping for RecordMeta {
    fn elastic_mapping() -> serde_json::Value {
        serde_json::json!({
            "guid": {
                "type": "keyword"
            },
            "id": {
                "type": "keyword"
            },
            "typ": {
                "type": "keyword"
            },
        })
    }
}
