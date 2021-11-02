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
        let guids = job.subjects.clone();
        let guids: Vec<&str> = guids.iter().map(|s| s.as_str()).collect();

        let job = ocypod::JobCreate::with_tags(job.args, job.subjects);
        let job_id = self.client.put_job(&typ, job).await?;

        // Load subjects and save the job id for each subject.
        // TODO: This should be somehow atomic.
        let mut records = self.db.get_many_records_untyped(&guids[..]).await?;
        for record in records.iter_mut() {
            record.meta_mut().insert_job(&typ, job_id);
        }
        self.db.put_untyped_record_bulk_update(records).await?;

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
        let changed_guids = if let Some(patches) = req.patches {
            self.db.apply_patches(patches).await?
        } else {
            vec![]
        };
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
