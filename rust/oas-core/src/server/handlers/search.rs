use crate::server::error::AppError;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{get, post};
use rocket_okapi::openapi;

static SEARCH_METHODS: &[&str; 2] = &["_search", "_msearch"];

#[openapi(skip)]
#[get("/search2")]
pub async fn search2(
    state: &rocket::State<crate::State>,
) -> Result<Json<serde_json::Value>, AppError> {
    use elasticsearch_dsl::*;

    let feed = "oas.Feed_9kmgrfvcz7p6k994vdddm1gb0g";

    let search = Search::new()
        .source(false)
        .from(0)
        .size(10)
        .stats("boolean-query")
        .query(Query::bool().must(Query::term("feed", feed)));
    // .filter(Query::term("tags", "production"))
    // .must_not(Query::range("age").gte(10).lte(10))
    // .shoulds([Query::term("tags", "env1"), Query::term("tags", "deployed")])
    // .minimum_should_match("1")
    // .boost(1),

    println!("query: {}", serde_json::to_string_pretty(&search).unwrap());
    let res = state
        .index_manager
        .post_index()
        .index()
        .query_records(search)
        .await?;
    let res = serde_json::to_value(res)?;
    Ok(Json(res))
}

#[openapi(skip)]
#[post("/search/<index_name>/<search_method>", data = "<body>")]
pub async fn search(
    state: &rocket::State<crate::State>,
    index_name: String,
    search_method: String,
    body: String,
) -> Result<String, AppError> {
    if &index_name != "oas" {
        return Err(AppError::Http(
            Status::BadRequest,
            "Invalid index name".into(),
        ));
    }

    if !SEARCH_METHODS.contains(&search_method.as_str()) {
        return Err(AppError::Http(
            Status::BadRequest,
            "Invalid search method".into(),
        ));
    }

    let index = &state.index_manager.post_index();
    let client = &index.client();

    let path = format!("{}/{}", index.name(), search_method);
    let res = client
        .send::<_, String>(
            elasticsearch::http::Method::Post,
            &path,
            elasticsearch::http::headers::HeaderMap::new(),
            None,
            Some(body),
            None,
        )
        .await?;

    let string = res.text().await?;
    Ok(string)
}
