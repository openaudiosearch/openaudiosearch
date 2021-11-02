use json_patch::Patch;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, time::Duration};

use crate::couch::CouchDB;

pub mod bin;
pub mod changes;
mod ocypod;
pub mod typs;

pub use ocypod::{
    JobFilter, JobId, JobInfo, JobInput, JobOutput, JobStatus, OcypodClient, DEFAULT_OCYPOD_URL,
};

#[derive(Debug, Clone)]
pub struct JobManager {
    client: OcypodClient,
    db: CouchDB,
}

impl JobManager {
    pub fn new(db: CouchDB, base_url: impl ToString) -> Self {
        let client = OcypodClient::new(base_url.to_string());
        Self { db, client }
    }

    pub(super) fn client(&self) -> &OcypodClient {
        &self.client
    }

    pub async fn create_job(&self, job: JobCreateRequest) -> anyhow::Result<ocypod::JobId> {
        let typ = job.typ;
        let queues = self.client.get_queues().await?;
        if !queues.contains(&typ) {
            // TODO: Make queue settings adjustable.
            let queue = ocypod::Queue {
                timeout: Duration::from_secs(3600).into(),
                heartbeat_timeout: Duration::from_secs(3600).into(),
                expires_after: Duration::from_secs(86400).into(),
                retries: 5,
                retry_delays: vec![],
            };
            self.client.put_queue(&typ, queue).await?;
        }

        // Push the job onto the job queue.
        let job = ocypod::JobCreate::with_tags(job.args, job.subjects);
        let job_id = self.client.put_job(&typ, job).await?;

        Ok(job_id)
    }

    pub async fn next_job(&self, typ: &str) -> anyhow::Result<Option<JobRequest>> {
        let input = self.client.next_job(typ).await?;
        if let Some(input) = input {
            let job = self.client.get_job(input.id).await?;
            let job_request = JobRequest {
                id: input.id,
                typ: job.queue,
                args: input.input,
                subjects: job.tags,
            };
            Ok(Some(job_request))
        } else {
            Ok(None)
        }
    }

    pub async fn job(&self, id: JobId) -> anyhow::Result<JobInfo> {
        self.client.get_job(id).await
    }

    pub async fn all_jobs(&self, filter: Option<JobFilter>) -> anyhow::Result<Vec<JobInfo>> {
        self.client.all_jobs(filter).await
    }

    pub async fn pending_jobs(&self, tag: &str, typ: &str) -> anyhow::Result<Vec<JobInfo>> {
        let filter = JobFilter::default()
            .with_queue(typ)
            .with_status(JobStatus::Queued);
        let pending_jobs = self.client().load_tag_filtered(tag, filter).await?;
        Ok(pending_jobs)
    }

    pub async fn set_progress(&self, job_id: JobId, req: JobProgressRequest) -> anyhow::Result<()> {
        let output = JobOutput {
            error: None,
            progress: Some(req.progress),
            meta: req.meta,
            duration: None,
        };
        self.client.update_job(job_id, None, Some(output)).await
    }

    pub async fn set_failed(&self, job_id: JobId, req: JobFailedRequest) -> anyhow::Result<()> {
        let status = JobStatus::Failed;
        let output = JobOutput {
            error: Some(req.error),
            progress: Some(100.0),
            meta: req.meta,
            duration: req.duration,
        };
        self.client
            .update_job(job_id, Some(status), Some(output))
            .await
    }

    pub async fn set_completed(
        &self,
        job_id: ocypod::JobId,
        req: JobCompletedRequest,
    ) -> anyhow::Result<()> {
        let job = self.job(job_id).await?;
        let mut patches = req.patches.unwrap_or_default();
        // Insert all tagged records into the changes to later
        // add the completed job id.
        ensure_entries(&mut patches, &job.tags);
        let tags = job.tags.clone();
        let typ = job.queue.clone();
        let changed_guids = self
            .db
            .apply_patches_with_callback(patches, move |records| {
                for record in records
                    .iter_mut()
                    .filter(|r| tags.iter().any(|tag| tag == r.guid()))
                {
                    record.meta_mut().jobs_mut().insert_completed(&typ, job_id);
                }
            })
            .await?;
        log::debug!(
            "Job {} completed with {} patches ({}) and meta `{}`",
            job_id,
            changed_guids.len(),
            changed_guids.join(", "),
            serde_json::to_string(&req.meta).unwrap_or_default()
        );
        let status = ocypod::JobStatus::Completed;
        let output = JobOutput {
            progress: Some(100.0),
            error: None,
            meta: req.meta,
            duration: req.duration,
        };
        let res = self
            .client
            .update_job(job_id, Some(status), Some(output))
            .await;
        res?;
        Ok(())
    }
}

fn ensure_entries(
    patches: &mut HashMap<String, Patch>,
    guids: impl IntoIterator<Item = impl AsRef<str>>,
) {
    // Insert all tagged records into the changes to later
    // add the completed job id.
    for tag in guids.into_iter() {
        if !patches.contains_key(tag.as_ref()) {
            patches.insert(tag.as_ref().to_string(), Patch(vec![]));
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JobCreateRequest {
    pub typ: String,
    pub args: serde_json::Value,
    pub subjects: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JobRequest {
    pub id: ocypod::JobId,
    pub typ: String,
    pub args: serde_json::Value,
    pub subjects: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JobCompletedRequest {
    pub patches: Option<HashMap<String, Patch>>,
    pub meta: Option<HashMap<String, String>>,
    pub duration: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JobProgressRequest {
    pub progress: f32,
    pub meta: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JobFailedRequest {
    #[serde(default)]
    pub error: serde_json::Value,
    #[serde(default)]
    pub meta: Option<HashMap<String, String>>,
    pub duration: Option<f32>,
}
