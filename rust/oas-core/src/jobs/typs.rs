use oas_common::{types::Media, types::Post, Record};
use serde_json::json;

use super::JobCreateRequest;

pub const ASR: &str = "asr";
pub const NLP: &str = "nlp";

pub fn asr_job(media: &Record<Media>) -> JobCreateRequest {
    JobCreateRequest {
        typ: "asr".to_string(),
        args: json!({ "media_id": media.id().to_string() }),
        subjects: vec![media.guid().to_string()],
    }
}

pub fn nlp_job(post: &Record<Post>) -> JobCreateRequest {
    JobCreateRequest {
        typ: "nlp".into(),
        args: json!({ "post_id": post.id().to_string() }),
        subjects: vec![post.guid().to_string()],
    }
}
