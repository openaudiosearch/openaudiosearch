use oas_common::{types::Media, types::Post, Record};
use serde_json::json;

use super::JobCreateRequest;

pub const ASR: &str = "asr";
pub const NLP: &str = "nlp";

pub fn asr_job(record: &Record<Media>) -> JobCreateRequest {
    JobCreateRequest {
        typ: ASR.to_owned(),
        args: json!({ "media_id": record.id().to_string() }),
        subjects: vec![record.guid().to_string()],
    }
}

pub fn nlp_job(record: &Record<Post>) -> JobCreateRequest {
    JobCreateRequest {
        typ: NLP.to_owned(),
        args: json!({ "post_id": record.id().to_string() }),
        subjects: vec![record.guid().to_string()],
    }
}
