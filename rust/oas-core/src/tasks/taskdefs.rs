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
