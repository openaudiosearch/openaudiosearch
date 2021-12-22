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
        let job = ocypod::JobCreate::with_tags(job.args, job.subjects);
        let job_id = self.client.put_job(&typ, job).await?;

        Ok(job_id)
    }

    pub async fn delete_job(&self, job: JobId) -> anyhow::Result<()> {
        self.client.delete_job(job).await?;
        Ok(())
    }
    pub async fn take_job_timeout(
        &self,
        typ: &str,
        timeout: JobTakeTimeout,
    ) -> anyhow::Result<Option<JobRequest>> {
        let start = std::time::Instant::now();
        let poll_interval = std::time::Duration::from_secs(1);
        let mut interval = tokio::time::interval(poll_interval);
        loop {
            let job = self.take_job(typ).await?;
            match job {
                Some(job) => return Ok(Some(job)),
                None => {
                    if let JobTakeTimeout::Timeout(timeout) = timeout {
                        if start.elapsed() > timeout {
                            return Ok(None);
                        }
                    }
                    interval.tick().await;
                }
            }
        }
    }

    pub async fn take_job(&self, typ: &str) -> anyhow::Result<Option<JobRequest>> {
        let input = self.client.take_job(typ).await?;
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

    pub async fn fetch_jobs(&self, filter: Option<JobFilter>) -> anyhow::Result<Vec<JobInfo>> {
        let filter = filter.unwrap_or_default();
        self.client.fetch_filtered(filter).await
    }

    pub async fn pending_jobs(&self, tag: &str, typ: &str) -> anyhow::Result<Vec<JobInfo>> {
        let filter = JobFilter::all()
            .with_tag(tag.to_string())
            .with_queue(typ)
            .with_status(JobStatus::Queued)
            .with_status(JobStatus::Running);
        let pending_jobs = self.client.fetch_filtered(filter).await?;
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
        let patches = req.patches.unwrap_or_default();
        let changed_guids = self.db.apply_patches(patches).await?;
        log::debug!(
            "Job {} completed with {} patches ({}) and meta `{}`",
            job_id,
            changed_guids.len(),
            changed_guids.join(", "),
            serde_json::to_string(&req.meta).unwrap_or_default()
        );
        let output = JobOutput {
            progress: Some(100.0),
            error: None,
            meta: req.meta,
            duration: req.duration,
        };

        let job = self.client.get_job(job_id).await?;
        self.on_complete(&job).await?;

        self.client
            .update_job(job_id, Some(JobStatus::Completed), Some(output))
            .await?;

        Ok(())
    }

    async fn on_complete(&self, job: &JobInfo) -> anyhow::Result<()> {
        let typ = &job.queue;
        match typ.as_str() {
            typs::ASR => typs::on_asr_complete(&self.db, &self, &job).await?,
            _ => {}
        }
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum JobTakeTimeout {
    Timeout(std::time::Duration),
    Forever,
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
