use crate::couch::types::PutResponse;
use crate::server::error::AppError;
use crate::State;
use oas_common::Record;
use rocket::http::Status;
use rocket::serde::json::Json;

use oas_common::util;
use rocket::{post, routes, Route};

use oas_common::types;

#[post("/", data = "<body>")]
async fn feed(
    state: &rocket::State<State>,
    body: Json<types::Feed>,
) -> Result<Json<PutResponse>, AppError> {
    // rocket::debug!("url: {}", body.into_inner().url);
    let feed = body.into_inner();
    let feed = Record::from_id_and_value(util::id_from_hashed_string(&feed.url), feed);
    let result = state.db.put_record(feed).await?;

    Ok(Json(result))
}

pub fn routes() -> Vec<Route> {
    routes![feed]
}
