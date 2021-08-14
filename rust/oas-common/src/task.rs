use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::TypedValue;

pub type TaskId = String;

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "state")]
pub enum TaskState {
    None,
    Wanted,
    // WaitingFor(OtherTaskToFinish),
    Running(TaskRunningState),
    Finished(TaskFinishedState),
}

impl std::default::Default for TaskState {
    fn default() -> Self {
        Self::None
    }
}

pub struct TaskFinishedModel {
    pub state: TaskFinishedState,
    pub result: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TaskFinishedState {
    pub task_id: String,
    #[serde(default)]
    pub success: bool,
    pub error: Option<String>,
    pub start: Option<DateTime<Utc>>,
    pub end: Option<DateTime<Utc>>,
    // Time in seconds
    #[serde(default)]
    pub took: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TaskRunningState {
    pub task_id: String,
    pub start: DateTime<Utc>,
}

pub trait TaskObject: TypedValue {
    type TaskStates: std::fmt::Debug;
    fn task_states(&self) -> Option<&Self::TaskStates> {
        None
    }
    fn task_states_mut(&mut self) -> Option<&mut Self::TaskStates> {
        None
    }
}
