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
    #[serde(default, skip_serializing_if = "SettingsMap::is_empty")]
    settings: SettingsMap,
}

impl JobsLog {
    pub fn is_empty(self) -> bool {
        self.settings.is_empty()
    }

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
}
