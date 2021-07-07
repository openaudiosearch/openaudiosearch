use oas_common::types::{Media, Feed};
use oas_common::{Record, TypedValue, UntypedRecord};
use rocket::serde::json::Json;
use rocket::{get, post, put, routes, Route};
use serde_json::Value;

use crate::couch::Doc;
use crate::server::error::{AppError, Result};

pub fn routes() -> Vec<Route> {
    routes![get_record, post_record, put_media]
}

#[put("/media/<id>", data = "<value>")]
async fn put_media(
    state: &rocket::State<crate::State>,
    id: String,
    value: Json<Media>,
) -> Result<serde_json::Value> {
    let record = Record::from_id_and_value(id, value.into_inner());
    state.db.put_record(record).await?;
    Ok(Value::Bool(true).into())
}

#[get("/<guid>")]
async fn get_record(state: &rocket::State<crate::State>, guid: String) -> Result<Doc> {
    let db = &state.db;
    let doc = db.get_doc(&guid).await?;
    Ok(doc.into())
}

#[post("/", data = "<record>")]
async fn post_record(
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
