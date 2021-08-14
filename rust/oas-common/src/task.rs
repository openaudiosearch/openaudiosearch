use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::TypedValue;

pub type TaskId = String;

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum TaskState {
    None,
    Wanted,
    // WaitingFor(OtherTaskToFinish),
    Running(TaskId),
    Finished(TaskId),
}

impl Default for TaskState {
    fn default() -> Self {
        Self::None
    }
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
