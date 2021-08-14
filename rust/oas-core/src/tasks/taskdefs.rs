#![allow(unused)]

use celery::task::TaskResult;
use oas_common::{
    types::{Media, Post},
    Record,
};
use serde_json::Value;

#[celery::task()]
pub fn transcribe(args: Value, opts: Value) -> TaskResult<Value> {
    Ok(Value::Null)
}

#[celery::task()]
pub fn download2(args: Record<Media>, opts: Value) -> TaskResult<Value> {
    Ok(Value::Null)
}

#[celery::task()]
pub fn nlp2(args: Record<Post>, opts: Value) -> TaskResult<Value> {
    Ok(Value::Null)
}

enum TaskNames {
    Transcribe,
    Nlp,
    Download,
}

mod new {
    use async_trait::async_trait;
    use celery::prelude::CeleryError;
    use oas_common::types::{Media, Post, Transcript};
    use oas_common::Record;
    use schemars::JsonSchema;
    use serde::de::DeserializeOwned;
    use serde::{Deserialize, Serialize};
    use std::fmt;
    use std::sync::Arc;

    use crate::tasks::CeleryManager;

    use super::nlp2;

    // pub struct TaskRepo {
    //     // tasks: HashMap<String,
    // }

    // impl TaskRepo {
    //     fn from_name_and_typ::<T: TaskObject>(&self, name: &str) {
    //         match name {
    //             NlpTask::NAME =>
    //         }
    //     }
    // }
    #[async_trait]
    pub trait Task {
        const NAME: &'static str;
        type Args: fmt::Debug + Serialize + DeserializeOwned + Send + 'static;
        type Opts: fmt::Debug + Serialize + DeserializeOwned + Send + 'static;
        type Output: fmt::Debug + Serialize + DeserializeOwned + Send + 'static;
        async fn run_celery(
            &self,
            _celery: Arc<CeleryManager>,
            _args: Self::Args,
            _opts: Self::Opts,
        ) -> Result<celery::task::AsyncResult, CeleryError> {
            Err(CeleryError::NoQueueToConsume)
        }
    }

    #[derive(Deserialize, Serialize, Debug, Clone, JsonSchema)]
    pub struct NlpOutput(serde_json::Value);

    struct NlpTask;

    #[async_trait]
    impl Task for NlpTask {
        const NAME: &'static str = "nlp";
        type Args = Record<Post>;
        type Opts = serde_json::Value;
        type Output = NlpOutput;
        async fn run_celery(
            &self,
            celery: Arc<CeleryManager>,
            args: Self::Args,
            opts: Self::Opts,
        ) -> Result<celery::task::AsyncResult, CeleryError> {
            celery.send_task(nlp2::new(args, opts)).await
        }
    }

    #[derive(Deserialize, Serialize, Debug, Clone, JsonSchema)]
    struct AsrTask;

    #[derive(Deserialize, Serialize, Debug, Clone, JsonSchema, Default)]
    struct AsrOpts {
        engine: Option<String>,
        default_language: Option<String>,
    }

    impl Task for AsrTask {
        const NAME: &'static str = "asr";
        type Args = Record<Media>;
        type Opts = AsrOpts;
        type Output = Transcript;
    }
}
