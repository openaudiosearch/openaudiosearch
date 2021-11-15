use oas_common::UntypedRecord;
use rocket::http::Status;
use rocket::post;
use rocket::response::status::Custom;
use rocket::serde::json::Json;
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::server::auth::AdminUser;
use crate::server::error::AppError;

/// Get record by GUID
// #[openapi(tag = "Changes")]
#[openapi(skip)]
#[post("/changes/durable/<token>")]
pub async fn durable_changes(
    _user: AdminUser,
    state: &rocket::State<crate::State>,
    token: String,
) -> Result<Custom<Json<Option<ChangesResponse>>>, AppError> {
    let opts = Default::default();
    let mut changes = state.db_manager.durable_changes(token, opts).await;
    let batch = changes.next().await?;
    let res = if let Some(batch) = batch {
        let res = ChangesResponse {
            changes: batch.records,
            seq: batch.last_seq.expect("Last seq may not be empty"),
        };
        // TODO: Make auto-ack optional
        changes.ack().await?;
        Ok(Custom(Status::Ok, Json(Some(res))))
    } else {
        Ok(Custom(Status::NoContent, Json(None)))
    };
    res
}

#[derive(Serialize, Deserialize, Debug, JsonSchema, Clone)]
pub struct ChangesResponse {
    pub changes: Vec<UntypedRecord>,
    pub seq: String,
}
