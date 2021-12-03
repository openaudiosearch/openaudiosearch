use oas_common::UntypedRecord;
use rocket::http::Status;
use rocket::post;
use rocket::response::status::Custom;
use rocket::serde::json::Json;
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::couch::durable_changes::ChangesOpts;
use crate::server::auth::AdminUser;
use crate::server::error::AppError;

/// Get record by GUID
///
/// This handler returns a "durable" changes stream of all records in the database.
/// The "token" is any string. On the first request, this will return the oldest batch
/// of changes. On subsequent requests with the same token, only changes that have not
/// been returned will be returned.
///
// #[openapi(tag = "Changes")]
#[openapi(skip)]
#[post("/changes/durable/<token>?<max_len>")]
pub async fn durable_changes(
    _user: AdminUser,
    state: &rocket::State<crate::State>,
    token: String,
    max_len: Option<usize>,
) -> Result<Custom<Json<Option<ChangesResponse>>>, AppError> {
    let opts = ChangesOpts::default().set_infinite(false);
    let opts = match max_len {
        Some(max_len) => opts.set_max_batch_len(max_len),
        None => opts,
    };
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
