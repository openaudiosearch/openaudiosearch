use oas_common::{types::Media, types::Post, Record, TypedValue};
use serde_json::json;

use super::{JobCreateRequest, JobInfo, JobManager};
use crate::CouchDB;

pub const ASR: &str = "asr";
pub const NLP: &str = "nlp";

pub fn asr_job(record: &Record<Media>, opts: Option<serde_json::Value>) -> JobCreateRequest {
    let opts = opts.unwrap_or_else(|| job_setting(record, ASR));
    JobCreateRequest {
        typ: ASR.to_owned(),
        args: json!({ "media_id": record.id().to_string(), "opts": opts }),
        subjects: vec![record.guid().to_string()],
    }
}

pub fn nlp_job(record: &Record<Post>, opts: Option<serde_json::Value>) -> JobCreateRequest {
    let opts = opts.unwrap_or_else(|| job_setting(record, ASR));
    JobCreateRequest {
        typ: NLP.to_owned(),
        args: json!({ "post_id": record.id().to_string(), "opts": opts }),
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
                let req = nlp_job(&post, None);
                let _job_id = jobs.create_job(req).await?;
            }
        }
    }
    Ok(())
}

fn job_setting<T: TypedValue>(record: &Record<T>, typ: &str) -> serde_json::Value {
    record
        .meta()
        .jobs()
        .setting(typ)
        .map(|x| x.clone())
        .unwrap_or_else(|| serde_json::Value::Object(Default::default()))
}
