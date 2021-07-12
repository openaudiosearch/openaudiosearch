use oas_common::types::Media;
use oas_common::{util, Record, TypedValue};
use rocket::serde::json::Json;
use rocket::{get, patch, post, put, routes, Route};
use serde_json::Value;

use crate::couch::PutResponse;
use crate::server::error::Result;

pub fn routes() -> Vec<Route> {
    routes![get_media, post_media, put_media, patch_media]
}

#[get("/<id>")]
async fn get_media(state: &rocket::State<crate::State>, id: String) -> Result<Record<Media>> {
    let record = state.db.get_record(&Media::guid(&id)).await?;
    Ok(Json(record))
}

#[post("/", data = "<value>")]
async fn post_media(
    state: &rocket::State<crate::State>,
    value: Json<Media>,
) -> Result<PutResponse> {
    let value = value.into_inner();
    let record = Record::from_id_and_value(util::id_from_hashed_string(&value.content_url), value);
    let res = state.db.put_record(record).await?;
    Ok(Json(res))
}

#[put("/<id>", data = "<value>")]
async fn put_media(
    state: &rocket::State<crate::State>,
    id: String,
    value: Json<Media>,
) -> Result<PutResponse> {
    let (typ, id) = util::split_and_check_guid::<Media>(&id)?;
    let record = Record::from_id_and_value(id, value.into_inner());
    let res = state.db.put_record(record).await?;
    Ok(Json(res))
}

#[patch("/<id>", data = "<value>")]
async fn patch_media(
    state: &rocket::State<crate::State>,
    id: String,
    value: Json<Value>,
) -> Result<PutResponse> {
    let db = &state.db;
    let guid = Media::guid(&id);
    let mut existing = db.get_doc(&guid).await?.into_untyped_record()?;
    existing.merge_json_value(value.into_inner())?;
    let record = existing.into_typed_record::<Media>()?;
    let res = db.put_record(record).await?;
    Ok(Json(res))
}
