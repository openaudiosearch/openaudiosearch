use oas_common::types::Post;
use oas_common::{util, Record, TypedValue};
use rocket::serde::json::Json;
use rocket::{get, patch, post, put};
use rocket_okapi::openapi;
use serde_json::Value;

use crate::couch::{PutResponse, PutResult};
use crate::server::auth::AdminUser;
use crate::server::error::Result;

/// Get a post record by id.
#[openapi(tag = "Post")]
#[get("/post/<id>")]
pub async fn get_post(state: &rocket::State<crate::State>, id: String) -> Result<Record<Post>> {
    let mut record = state.db.get_record(&Post::guid(&id)).await?;
    let _ = record.resolve_refs(&state.db).await;
    Ok(Json(record))
}

/// Create a new post record
#[openapi(tag = "Post")]
#[post("/post", data = "<value>")]
pub async fn post_post(
    _user: AdminUser,
    state: &rocket::State<crate::State>,
    value: Json<Post>,
) -> Result<PutResponse> {
    let mut value = value.into_inner();
    let id = value.identifier.unwrap_or_else(util::id_from_uuid);
    value.identifier = Some(id.clone());
    let record = Record::from_id_and_value(id, value);
    let res = state.db.put_record(record).await?;
    Ok(Json(res))
}

#[openapi(tag = "Post")]
#[post("/post?batch=1", data = "<value>")]
pub async fn post_post_batch(
    _user: AdminUser,
    state: &rocket::State<crate::State>,
    value: Json<Vec<Post>>,
) -> Result<Vec<PutResult>> {
    let values = value.into_inner();
    let records = values
        .into_iter()
        .map(|mut value| {
            let id = value.identifier.unwrap_or_else(util::id_from_uuid);
            value.identifier = Some(id.clone());
            let record = Record::from_id_and_value(id, value);
            record
        })
        .collect::<Vec<_>>();
    let res = state.db.put_record_bulk(records).await?;
    Ok(Json(res))
}

/// Put (update & overwrite) a post record
#[openapi(tag = "Post")]
#[put("/post/<id>", data = "<value>")]
pub async fn put_post(
    _user: AdminUser,
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
    _user: AdminUser,
    state: &rocket::State<crate::State>,
    id: String,
    value: Json<Value>,
) -> Result<PutResponse> {
    let db = &state.db;
    let guid = Post::guid(&id);

    let patch: json_patch::Patch = serde_json::from_value(value.into_inner())?;
    let mut existing = db.get_doc(&guid).await?.into_untyped_record()?;

    existing.apply_json_patch(&patch)?;
    let record = existing.into_typed_record::<Post>()?;
    let res = db.put_record(record).await?;
    Ok(Json(res))
}
