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

mod new {
    use std::fmt;

    use oas_common::types::Post;
    use oas_common::Record;
    use schemars::JsonSchema;
    use serde::de::DeserializeOwned;
    use serde::{Deserialize, Serialize};
    pub trait Task {
        type Args: fmt::Debug + Serialize + DeserializeOwned;
        type Opts: fmt::Debug + Serialize + DeserializeOwned;
        type Result: fmt::Debug + Serialize + DeserializeOwned;
    }

    #[derive(Deserialize, Serialize, Debug, Clone, JsonSchema)]
    pub struct NlpResult(serde_json::Value);

    struct NlpTask;

    impl Task for NlpTask {
        type Args = Record<Post>;
        type Opts = ();
        type Result = NlpResult;
    }
}
