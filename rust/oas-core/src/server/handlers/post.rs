use oas_common::types::Post;
use oas_common::{util, Record, TypedValue};
use rocket::serde::json::Json;
use rocket::{get, patch, post, put};
use rocket_okapi::openapi;
use serde_json::Value;

use crate::couch::PutResponse;
use crate::server::error::Result;

/// Get a post record by id.
#[openapi(tag = "Post")]
#[get("/post/<id>")]
pub async fn get_post(state: &rocket::State<crate::State>, id: String) -> Result<Record<Post>> {
    let record = state.db.get_record(&Post::guid(&id)).await?;
    Ok(Json(record))
}

/// Create a new post record
#[openapi(tag = "Post")]
#[post("/post", data = "<value>")]
pub async fn post_post(
    state: &rocket::State<crate::State>,
    value: Json<Post>,
) -> Result<PutResponse> {
    let mut value = value.into_inner();
    let id = value.identifier.unwrap_or_else(|| util::id_from_uuid());
    value.identifier = Some(id.clone());
    let record = Record::from_id_and_value(id, value);
    let res = state.db.put_record(record).await?;
    Ok(Json(res))
}

/// Put (update & overwrite) a post record
#[openapi(tag = "Post")]
#[put("/post/<id>", data = "<value>")]
pub async fn put_post(
    state: &rocket::State<crate::State>,
    id: String,
    value: Json<Post>,
) -> Result<PutResponse> {
    let (_typ, id) = util::split_and_check_guid::<Post>(&id)?;
    let record = Record::from_id_and_value(id, value.into_inner());
    let res = state.db.put_record(record).await?;
    Ok(Json(res))
}

/// Patch (update) a post record.
#[openapi(tag = "Post")]
#[patch("/post/<id>", data = "<value>")]
pub async fn patch_post(
    state: &rocket::State<crate::State>,
    id: String,
    value: Json<Value>,
) -> Result<PutResponse> {
    let db = &state.db;
    let guid = Post::guid(&id);
    let mut existing = db.get_doc(&guid).await?.into_untyped_record()?;
    existing.merge_json_value(value.into_inner())?;
    let record = existing.into_typed_record::<Post>()?;
    let res = db.put_record(record).await?;
    Ok(Json(res))
}
