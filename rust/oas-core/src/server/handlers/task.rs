use crate::server::error::AppError;
use crate::State;
use oas_common::{types::Media, TypedValue};
use rocket::serde::json::Json;
use rocket_okapi::openapi;

use rocket::post;

/// Create a new feed
#[openapi(tag = "Task")]
#[post("/task/transcribe-media/<id>")]
pub async fn post_transcribe_media(
    state: &rocket::State<State>,
    id: String,
) -> Result<Json<String>, AppError> {
    let media = state.db.get_record::<Media>(&Media::guid(&id)).await?;
    match state.tasks.transcribe_media(&media).await {
        Ok(task_id) => Ok(Json(task_id)),
        Err(err) => Err(AppError::Other(format!("{}", err))),
    }
}

// #[openapi(tag = "Task")]
// #[post("/task/result/media/<media_guid>/<task_id>")]
// pub async fn post_transcribe_media(
//     state: &rocket::State<State>,
//     guid: String,
// ) -> Result<Json<String>, AppError> {
// }

// #[openapi(tag = "Task")]
// #[post("/task/object/<id>")]
// pub async fn post_transcribe_media(
//     state: &rocket::State<State>,
//     id: String,
// ) -> Result<Json<String>, AppError> {
//     let media = state.db.get_record::<Media>(&Media::guid(&id)).await?;
//     match state.tasks.transcribe_media(&media).await {
//         Ok(task_id) => Ok(Json(task_id)),
//         Err(err) => Err(AppError::Other(format!("{}", err))),
//     }
// }

// #[openapi(tag = "Task")]
// #[post("/task/want/<task_name>/<guid>")]
// pub async fn post_want(state: &rocket::State<State>, task_name: String, guid: String) {
//     let record = state.db.get_record_untyped(&guid).await?;
//     record.task_states_mut().add(
//     // let patch = json_path::from_value(json!({
//     //     { "op":
//     // })
// //      from_str(r#"[
// //   { "op": "test", "path": "/0/name", "value": "Andrew" },
// //   { "op": "add", "path": "/0/happy", "value": true }
// // ]"#)
// //     let patch = json_patch::Patch::
//     // state.db.
// }

// #[openapi(tag = "Task")]
// #[post("/task/finish/<task_id>", data= "<data>")]
// pub async fn post_finish(state: &rocket::State<State>, task_id: String, data: Json<TaskFinishRequest>) {}

// #[openapi(tag = "Task")]
// #[post("/task/finish", data = "<data>")]
// pub async fn post_finish(
//     state: &rocket::State<State>,
//     task_id: String,
//     data: Json<TaskFinishRequest>,
// ) {
//     let db = state.db_manager.meta_db();
// }

// pub struct TaskWantedRequest {
//     task: String,
//     args_guid: Option<String>,
// }

// pub struct TaskFinishRequest {
//     task_id: String,
//     // Time in ms
//     took: u32,
//     success: bool,
//     error: Option<String>,
//     patches: HashMap<String, json_patch::Patch>,
//     meta: HashMap<String, serde_json::Value>, // tasks: HashMap<String, serde_json::Value>
// }

// #[openapi(tag = "Task")]
// #[post("/task/<id>")]
// pub async fn post_transcribe_media(
//     state: &rocket::State<State>,
//     id: String,
//     data: Json<serde_json::Value>,
// ) -> Result<Json<String>, AppError> {
//     let media = state.db.get_record::<Media>(&Media::guid(&id)).await?;
//     match state.tasks.transcribe_media(&media).await {
//         Ok(task_id) => Ok(Json(task_id)),
//         Err(err) => Err(AppError::Other(format!("{}", err))),
//     }
// }
