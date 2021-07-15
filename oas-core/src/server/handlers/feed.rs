use crate::couch::types::{Doc, PutResponse};
use crate::server::error::AppError;
use crate::State;
use oas_common::Record;
use rocket::http::Status;
use rocket::serde::json::Json;

use oas_common::util;
use rocket::{get, post, put, routes, Route};

use oas_common::types;

#[post("/", data = "<body>")]
async fn post_feed(
    state: &rocket::State<State>,
    body: Json<types::Feed>,
) -> Result<Json<PutResponse>, AppError> {
    // rocket::debug!("url: {}", body.into_inner().url);
    let feed = body.into_inner();
    let feed = Record::from_id_and_value(util::id_from_hashed_string(&feed.url), feed);
    let result = state.db.put_record(feed).await?;

    Ok(Json(result))
}

#[put("/<id>", data = "<body>")]
async fn put_feed(
    state: &rocket::State<State>,
    id: String,
    body: Json<types::Feed>,
) -> Result<Json<PutResponse>, AppError> {
    let feed = body.into_inner();
    let feed = Record::from_id_and_value(id, feed);
    let result = state.db.put_record(feed).await?;

    Ok(Json(result))
}

#[get("/<id>")]
async fn get_feed(
    state: &rocket::State<State>,
    id: String,
) -> Result<Json<Record<types::Feed>>, AppError> {
    let feed = state.db.get_record(&id).await?;
    Ok(Json(feed))
}

pub fn routes() -> Vec<Route> {
    routes![get_feed, post_feed, put_feed]
}
