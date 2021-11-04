use oas_common::{types::Media, types::Post, Record};
use serde_json::json;

use super::{JobCreateRequest, JobInfo, JobManager};
use crate::CouchDB;

pub const ASR: &str = "asr";
pub const NLP: &str = "nlp";

pub fn asr_job(record: &Record<Media>) -> JobCreateRequest {
    JobCreateRequest {
        typ: ASR.to_owned(),
        args: json!({ "media_id": record.id().to_string() }),
        subjects: vec![record.guid().to_string()],
    }
}

/// When an ASR job completes, create NLP jobs for all posts that contain this media.
pub async fn on_asr_complete(db: &CouchDB, jobs: &JobManager, job: &JobInfo) -> anyhow::Result<()> {
    let id = job.input.get("media_id").and_then(|id| match id {
        serde_json::Value::String(id) => Some(id.to_string()),
        _ => None,
    });
    if let Some(id) = id {
        let record = db.table::<Media>().get(&id).await?;
        for post_ref in record.value.posts.iter() {
            let post = db.table::<Post>().get(post_ref.id()).await;
            if let Ok(post) = post {
                let req = nlp_job(&post);
                let _job_id = jobs.create_job(req).await?;
            }
        }
    }
    Ok(())
}

pub fn nlp_job(record: &Record<Post>) -> JobCreateRequest {
    JobCreateRequest {
        typ: NLP.to_owned(),
        args: json!({ "post_id": record.id().to_string() }),
        subjects: vec![record.guid().to_string()],
    }
}
