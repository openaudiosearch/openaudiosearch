use oas_common::types::Media;
use oas_common::{util, Record};
use rocket::post;
use rocket::serde::json::Json;
use rocket_okapi::{openapi, JsonSchema};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::jobs::{typs as job_typs, JobCreateRequest};
use crate::server::auth::AdminUser;
use crate::server::error::Result;

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct TranscribeJobPayload {
    pub content_url: String,
    pub webhook_on_complete: Option<String>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct TranscribeJobResult {
    pub media_id: String,
    pub job_id: u64,
}

/// Create a new media record
#[openapi(tag = "Media")]
#[post("/transcribe", data = "<value>")]
pub async fn post_transcribe_job(
    _user: AdminUser,
    state: &rocket::State<crate::State>,
    value: Json<TranscribeJobPayload>,
) -> Result<TranscribeJobResult> {
    let value = value.into_inner();
    let media = Media {
        content_url: value.content_url,
        ..Default::default()
    };
    let record = Record::from_id_and_value(util::id_from_hashed_string(&media.content_url), media);
    let _res = state.db.put_record(record.clone()).await?;

    let job = JobCreateRequest {
        typ: job_typs::ASR.to_owned(),
        args: json!({ "media_id": record.id().to_string(), "webhook_on_complete": value.webhook_on_complete }),
        subjects: vec![record.guid().to_string()],
    };
    let job_id = state.jobs.create_job(job).await?;
    let res = TranscribeJobResult {
        job_id,
        media_id: record.id().to_owned(),
    };
    Ok(Json(res))
}
