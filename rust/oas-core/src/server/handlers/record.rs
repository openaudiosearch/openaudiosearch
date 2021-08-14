use oas_common::types::{Feed, Media};
use oas_common::{TypedValue, UntypedRecord};
use rocket::serde::json::Json;
use rocket::{get, post};
use rocket_okapi::openapi;
use serde_json::Value;

use crate::couch::Doc;
use crate::server::error::{AppError, Result};
use crate::server::auth::AdminUser;

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

    let record = record.into_inner();
    match record.typ() {
        Media::NAME => {
            let record = record.into_typed_record::<Media>()?;
            db.put_record(record).await?;
            Ok(Value::Bool(true).into())
        }
        Feed::NAME => {
            let record = record.into_typed_record::<Feed>()?;
            db.put_record(record).await?;
            Ok(Value::Bool(true).into())
        }
        _ => Err(AppError::Other("Unknown type".to_string())),
    }
}
