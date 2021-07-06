use oas_common::types::{AudioObject, Feed};
use oas_common::{DecodingError, Record, TypedValue, UntypedRecord};
use rocket::http::Status;
use rocket::response::status::Custom;
use rocket::response::{status, Responder};
use rocket::{get, post, put, response, response::content, routes, Request, Route};
use serde::Serialize;
use serde_json::{json, Value};
use thiserror::Error;

use crate::couch::{CouchError, Doc};
use crate::server::error::{AppError, Result};

pub fn routes() -> Vec<Route> {
    routes![get_record, post_record, put_media]
}

#[get("/<guid>")]
async fn get_record(state: &rocket::State<crate::State>, guid: String) -> Result<Doc> {
    let db = &state.db;
    let doc = db.get_doc(&guid).await?;
    Ok(doc.into())
}

#[put("/media/<id>", data = "<value>")]
async fn put_media(
    state: &rocket::State<crate::State>,
    id: String,
    value: rocket::serde::json::Json<AudioObject>,
) -> Result<serde_json::Value> {
    let record = Record::from_id_and_value(id, value.into_inner());
    state.db.put_record(record).await?;
    Ok(Value::Bool(true).into())
}

#[post("/", data = "<record>")]
async fn post_record(
    state: &rocket::State<crate::State>,
    record: rocket::serde::json::Json<UntypedRecord>,
) -> Result<serde_json::Value> {
    let db = &state.db;

    let record = record.into_inner();
    match record.typ() {
        AudioObject::NAME => {
            let record = record.into_typed_record::<AudioObject>()?;
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
