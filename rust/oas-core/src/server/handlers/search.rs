use crate::server::error::AppError;
use rocket::http::Status;
use rocket::post;
use rocket_okapi::openapi;

static SEARCH_METHODS: &[&str; 2] = &["_search", "_msearch"];

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
