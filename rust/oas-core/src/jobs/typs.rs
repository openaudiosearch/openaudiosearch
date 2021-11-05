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

mod foo {
    use crate::jobs::{JobCreateRequest, JobInfo, JobManager};
    use crate::CouchDB;
    use async_trait::async_trait;
    use oas_common::{types::Media, types::Post, Record};
    // use serde::de::DeserializeOwned;
    // use serde::Serialize;
    use serde_json::json;
    use std::collections::HashMap;
    use thiserror::Error;
    #[derive(Error, Debug)]
    enum JobError {
        #[error("Hook not implemented.")]
        Unimplemented,
        #[error("{}", .0)]
        Error(#[from] anyhow::Error),
    }

    pub type Args = serde_json::Value;

    #[async_trait]
    trait JobBuilder: Send + Sync {
        fn name(&self) -> &str;
        async fn on_complete(
            &self,
            db: &CouchDB,
            jobs: &JobManager,
            job: &JobInfo,
        ) -> Result<(), JobError> {
            Err(JobError::Unimplemented)
        }
        fn scan_media(&self, record: &Record<Media>) -> Option<Args> {
            None
        }
        fn scan_post(&self, record: &Record<Post>) -> Option<Args> {
            None
        }

        fn from_post(&self, record: &Record<Post>) -> Option<JobCreateRequest> {
            self.scan_post(&record).map(|args| JobCreateRequest {
                typ: self.name().to_string(),
                args,
                subjects: vec![record.guid().to_string()],
            })
        }
        fn from_media(&self, record: &Record<Media>) -> Option<JobCreateRequest> {
            self.scan_media(&record).map(|args| JobCreateRequest {
                typ: self.name().to_string(),
                args,
                subjects: vec![record.guid().to_string()],
            })
        }
    }

    #[async_trait]
    impl JobBuilder for Box<dyn JobBuilder> {
        fn name(&self) -> &str {
            self.name()
        }
        async fn on_complete(
            &self,
            db: &CouchDB,
            jobs: &JobManager,
            job: &JobInfo,
        ) -> Result<(), JobError> {
            self.on_complete(db, jobs, job).await
        }
        fn scan_media(&self, record: &Record<Media>) -> Option<Args> {
            self.scan_media(record)
        }
        fn scan_post(&self, record: &Record<Post>) -> Option<Args> {
            self.scan_post(record)
        }
    }

    struct World {
        jobs: HashMap<String, Box<dyn JobBuilder>>,
    }

    impl World {
        pub fn register_job(&mut self, job: Box<dyn JobBuilder>) {
            self.jobs.insert(job.name().into(), job);
        }

        pub fn get(&self, name: &str) -> Option<&dyn JobBuilder> {
            self.jobs.get(name).map(Box::as_ref)
        }

        pub fn on_media(&self, record: &Record<Media>) -> Vec<JobCreateRequest> {
            let subjects = vec![record.guid().to_owned()];
            self.build_with(subjects, |job| job.scan_media(&record))
        }

        fn build_with<F>(&self, subjects: Vec<String>, closure: F) -> Vec<JobCreateRequest>
        where
            F: Fn(&dyn JobBuilder) -> Option<Args>,
        {
            let reqs = self
                .jobs
                .values()
                .filter_map(|job| {
                    let args = (closure)(job);
                    args.map(|args| JobCreateRequest {
                        typ: job.name().to_string(),
                        args,
                        subjects: subjects.clone(),
                    })
                })
                .collect();
            reqs
        }
    }

    fn register_jobs(world: &mut World) {
        world.register_job(Box::new(AsrJob));
        world.register_job(Box::new(NlpJob));
    }

    struct NlpJob;
    #[async_trait]
    impl JobBuilder for NlpJob {
        fn name(&self) -> &str {
            "asr"
        }
        fn scan_post(&self, record: &Record<Post>) -> Option<Args> {
            Some(json!({ "media_id": record.id().to_string() }))
        }
    }

    struct AsrJob;
    #[async_trait]
    impl JobBuilder for AsrJob {
        fn name(&self) -> &str {
            "asr"
        }
        fn scan_media(&self, record: &Record<Media>) -> Option<Args> {
            Some(json!({ "media_id": record.id().to_string() }))
        }
        async fn on_complete(
            &self,
            db: &CouchDB,
            jobs: &JobManager,
            job: &JobInfo,
        ) -> Result<(), JobError> {
            let id = match job.input.get("media_id") {
                Some(serde_json::Value::String(id)) => Some(id.to_string()),
                _ => None,
            };
            if let Some(id) = id {
                let record = db
                    .table::<Media>()
                    .get(&id)
                    .await
                    .map_err(|err| anyhow::anyhow!(err))?;

                futures::future::join_all(record.value.posts.iter().map(|post_ref| {
                    let table = db.table::<Post>();
                    async move {
                        let req = table
                            .get(post_ref.id())
                            .await
                            .ok()
                            .and_then(|record| NlpJob.from_post(&record));
                        if let Some(req) = req {
                            // TODO: Handle error.
                            let _ = jobs.create_job(req).await;
                        }
                    }
                }))
                .await;
            }
            Ok(())
        }
    }
}
