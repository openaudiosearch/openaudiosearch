use crate::server::error::AppError;
use rocket::http::Status;
use rocket::{post, routes, Route};
use rocket_okapi::openapi;

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

    let index = &state.index;
    let client = index.client();

    let path = format!("{}/{}", index.index(), search_method);
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
