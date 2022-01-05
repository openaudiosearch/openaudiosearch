use oas_common::types::{Feed, Media, Post};
use oas_common::{TypedValue, UntypedRecord};
use rocket::serde::json::Json;
use rocket::{get, post};
use rocket_okapi::openapi;

use crate::couch::Doc;
use crate::server::auth::AdminUser;
use crate::server::error::{AppError, Result};

// pub fn routes() -> Vec<Route> {
//     routes![get_record, post_record]
// }

/// Get record by GUID
// #[openapi(tag = "Record")]
#[openapi(skip)]
#[get("/record/<guid>")]
pub async fn get_record(state: &rocket::State<crate::State>, guid: String) -> Result<Doc> {
    let db = &state.db;
    let doc = db.get_doc(&guid).await?;
    Ok(doc.into())
}

/// Post a new record
// #[openapi(tag = "Record")]
#[openapi(skip)]
#[post("/record", data = "<record>")]
pub async fn post_record(
    _user: AdminUser,
    state: &rocket::State<crate::State>,
    record: Json<UntypedRecord>,
) -> Result<serde_json::Value> {
    let db = &state.db;

    let mut records = vec![];
    let record = record.into_inner();
    match record.typ() {
        Post::NAME => {
            let mut record = record.into_typed_record::<Post>()?;
            let mut refs = record.extract_refs();
            records.append(&mut refs);
            records.push(record.into_untyped()?);
        }
        Media::NAME => {
            let record = record.into_typed_record::<Media>()?;
            records.push(record.into_untyped()?);
        }
        Feed::NAME => {
            let record = record.into_typed_record::<Feed>()?;
            records.push(record.into_untyped()?);
        }
        _ => return Err(AppError::Other("Unknown type".to_string())),
    }

    let res = db.put_untyped_record_bulk_update(records).await?;
    let json_res = serde_json::to_value(res)?;
    Ok(Json(json_res))
}
