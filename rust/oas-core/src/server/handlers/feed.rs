use crate::couch::types::PutResponse;
use crate::server::error::AppError;
use crate::State;
use oas_common::util;
use oas_common::{types, Record, TypedValue};
use rocket::serde::json::Json;
use rocket::{get, post, put};
use rocket_okapi::openapi;

/// Create a new feed
#[openapi(tag = "Feed")]
#[post("/feed", data = "<body>")]
pub async fn post_feed(
    state: &rocket::State<State>,
    body: Json<types::Feed>,
) -> Result<Json<PutResponse>, AppError> {
    // rocket::debug!("url: {}", body.into_inner().url);
    let feed = body.into_inner();
    let feed = Record::from_id_and_value(util::id_from_hashed_string(&feed.url), feed);
    match feed.validate() {
        Ok(_) => {
            let result = state.db.put_record(feed).await?;
            Ok(Json(result))
        }
        Err(err) => Err(AppError::ValidationError(err)),
    }
}

/// Put feed info by feed ID
#[openapi(tag = "Feed")]
#[put("/feed/<id>", data = "<body>")]
pub async fn put_feed(
    state: &rocket::State<State>,
    id: String,
    body: Json<types::Feed>,
) -> Result<Json<PutResponse>, AppError> {
    let feed = body.into_inner();
    let feed = Record::from_id_and_value(id, feed);
    let result = state.db.put_record(feed).await?;

    Ok(Json(result))
}

/// Get a feed by its id
#[openapi(tag = "Feed")]
#[get("/feed/<id>")]
pub async fn get_feed(
    state: &rocket::State<State>,
    id: String,
) -> Result<Json<Record<types::Feed>>, AppError> {
    let feed = state.db.get_record(&types::Feed::guid(&id)).await?;
    Ok(Json(feed))
}

/// Get all feeds
#[openapi(tag = "Feed")]
#[get("/feed")]
pub async fn get_feeds(
    state: &rocket::State<State>,
) -> Result<Json<Vec<Record<types::Feed>>>, AppError> {
    let feeds = state.db.get_all_records().await?;
    Ok(Json(feeds))
}
