use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type JobId = u64;
pub type JobTyp = String;

pub type SettingsMap = HashMap<JobTyp, Option<serde_json::Value>>;

#[derive(Serialize, Deserialize, Debug, Clone, Default, JsonSchema)]
pub struct JobTypId {
    typ: String,
    id: JobId,
}

impl JobTypId {
    pub fn new(typ: String, id: JobId) -> Self {
        Self { typ, id }
    }

    pub fn typ(&self) -> &str {
        &self.typ
    }

    pub fn id(&self) -> JobId {
        self.id
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, JsonSchema)]
pub struct JobsLog {
    settings: SettingsMap,
    completed: Vec<JobTypId>,
    failed: Vec<JobTypId>,
}

impl JobsLog {
    pub fn settings(&self) -> &SettingsMap {
        &self.settings
    }

    pub fn settings_mut(&mut self) -> &mut SettingsMap {
        &mut self.settings
    }

    pub fn copy_settings(&mut self, settings: &SettingsMap) {
        for (key, value) in settings {
            self.settings.insert(key.to_string(), value.clone());
        }
    }

    pub fn insert_completed(&mut self, typ: &str, id: JobId) {
        let typ_id = JobTypId::new(typ.to_string(), id);
        self.completed.push(typ_id)
    }

    pub fn insert_failed(&mut self, typ: &str, id: JobId) {
        let typ_id = JobTypId::new(typ.to_string(), id);
        self.failed.push(typ_id)
    }

    pub fn completed(&self) -> &[JobTypId] {
        self.completed.as_slice()
    }

    pub fn failed(&self) -> &[JobTypId] {
        self.failed.as_slice()
    }
}

// #[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
// pub struct CompletedJob {
//     pub id: JobId,
//     pub typ: String,
//     #[serde(default)]
//     pub meta: HashMap<String, String>,
//     pub started: u64,
//     pub finished: u64,
// }

// impl JobOutput {
//     pub fn merge(&mut self, other: &JobOutput) {
//         if let Some(progress) = other.progress {
//             self.progress = Some(progress);
//         }
//         if let Some(error) = &other.error {
//             self.error = Some(error.clone());
//         }
//         if let Some(meta) = &other.meta {
//             if self.meta.is_none() {
//                 self.meta = Some(HashMap::new());
//             }
//             let meta_mut = self.meta.as_mut().unwrap();
//             for (key, value) in meta {
//                 meta_mut.insert(key.to_string(), value.to_string());
//             }
//         }
//     }
// }
// pub struct JobRequest {
//     typ: JobTyp,
//     opts: Option<serde_json::Value>,
// }

// impl JobRequest {
//     pub fn new(typ: String) -> Self {
//         Self { typ, opts: None }
//     }

//     pub fn with_opts(typ: String, opts: serde_json::Value) -> Self {
//         Self {
//             typ,
//             opts: Some(opts),
//         }
//     }
// }
// enum JobState {
//     Requesting,
//     Pending,
//     Completed,
//     Failed,
// }
