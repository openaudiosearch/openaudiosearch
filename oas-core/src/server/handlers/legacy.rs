// use rocket::response::status::InternalServerError;

use rocket::{post, response::content::Json, routes, Route};

mod types {
    use serde::{Deserialize, Serialize};

    use crate::tasks::Engine;

    #[derive(Debug, Serialize, Deserialize)]
    pub struct TranscriptRequest {
        pub media_url: String,
        pub engine: Engine,
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "lowercase")]
    pub enum TranscriptStatus {
        Queued,
        Processing,
        Completed,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct TranscriptResponse {
        pub id: String,
        pub status: TranscriptStatus,
    }
}

pub type Result<T> = std::result::Result<T, Json<String>>;

#[post("/transcript")]
async fn post_transcript() -> Result<Json<String>> {
    let res = types::TranscriptResponse {
        id: "foo".to_string(),
        status: types::TranscriptStatus::Queued,
    };
    Ok(Json(serde_json::to_string(&res).unwrap()))
}

pub fn routes() -> Vec<Route> {
    routes![post_transcript]
}
