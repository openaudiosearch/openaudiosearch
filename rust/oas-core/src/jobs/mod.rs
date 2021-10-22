use json_patch::Patch;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, time::Duration};

use crate::couch::CouchDB;

mod ocypod;
pub mod typs;
pub use ocypod::{JobInfo, JobInput, JobStatus, OcypodClient};

pub type JobId = u64;

#[derive(Debug, Clone)]
pub struct JobManager {
    client: OcypodClient,
    db: CouchDB,
}

impl JobManager {
    pub fn new(db: CouchDB, base_url: String) -> Self {
        let client = OcypodClient::new(base_url);
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

    pub async fn next_job(&self, typ: &str) -> anyhow::Result<JobRequest> {
        let input = self.client.next_job(&typ).await?;
        let job = self.client.get_job(input.id).await?;
        let job_request = JobRequest {
            id: input.id.clone(),
            typ: job.queue,
            args: input.input,
            subjects: job.tags,
        };
        Ok(job_request)
    }

    pub async fn get_job(&self, id: JobId) -> anyhow::Result<JobInfo> {
        self.client.get_job(id).await
    }

    pub async fn apply_results(
        &self,
        job_id: ocypod::JobId,
        results: JobResults,
    ) -> anyhow::Result<()> {
        let patches = results.patches;
        let guids: Vec<&str> = patches.keys().map(|s| s.as_str()).collect();
        let records = self.db.get_many_records_untyped(&guids).await?;
        let mut changed_records = vec![];
        for mut record in records.into_iter() {
            if let Some(patch) = patches.get(record.guid()) {
                let success = record.apply_json_patch(&patch);
                if let Ok(_) = success {
                    changed_records.push(record)
                }
            }
        }
        self.db
            .put_untyped_record_bulk_update(changed_records)
            .await?;
        let status = Some(ocypod::JobStatus::Completed);
        let output = Some(serde_json::to_value(results.meta)?);
        let res = self.client.update_job(job_id, status, output).await;
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
pub struct JobResults {
    #[serde(default)]
    pub patches: HashMap<String, Patch>,
    #[serde(default)]
    pub meta: HashMap<String, String>,
}

// trait JobTyp {
//     const NAME: &'static str;
//     type Args: Serialize + DeserializeOwned;
// }
// mod jobs {
//     struct AsrJob;

// }
