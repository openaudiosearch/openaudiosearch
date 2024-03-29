use futures::future::join_all;
use http::StatusCode;
use serde::de::{Deserializer, Error};
use serde::ser::Serializer;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::time;

pub type JobId = u64;
pub const DEFAULT_OCYPOD_URL: &str = "http://localhost:8023";

#[derive(Clone, Debug)]
pub struct OcypodClient {
    base_url: String,
    client: reqwest::Client,
}

impl OcypodClient {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            client: reqwest::Client::new(),
        }
    }
    pub async fn put_queue(&self, name: &str, queue: Queue) -> anyhow::Result<()> {
        let url = format!("{}/queue/{}", self.base_url, name);
        let res = self.client.put(&url).json(&queue).send().await?;
        check_response(&res)?;
        Ok(())
    }

    pub async fn put_job(&self, queue: &str, job: JobCreate) -> anyhow::Result<JobId> {
        let url = format!("{}/queue/{}/job", self.base_url, queue);
        let res = self.client.post(&url).json(&job).send().await?;
        check_response(&res)?;
        let job_id: JobId = res.json().await?;
        Ok(job_id)
    }

    pub async fn take_job(&self, queue: &str) -> anyhow::Result<Option<JobInput>> {
        let url = format!("{}/queue/{}/job", self.base_url, queue);
        let res = self.client.get(&url).send().await?;
        if res.status() == StatusCode::NO_CONTENT || res.status() == StatusCode::NOT_FOUND {
            Ok(None)
        } else {
            check_response(&res)?;
            let body: JobInput = res.json().await?;
            Ok(Some(body))
        }
    }

    pub async fn update_job(
        &self,
        job_id: JobId,
        status: Option<JobStatus>,
        output: Option<JobOutput>,
    ) -> anyhow::Result<()> {
        let current = self.get_job(job_id).await?;
        let output = if let Some(output) = output {
            if let Some(mut old_output) = current.output {
                old_output.merge(&output);
                Some(old_output)
            } else {
                Some(output)
            }
        } else {
            None
        };

        let body = JobUpdate { output, status };

        let url = format!("{}/job/{}", self.base_url, job_id);
        let res = self.client.patch(&url).json(&body).send().await?;
        check_response(&res)?;
        Ok(())
    }

    pub async fn get_job(&self, job_id: JobId) -> anyhow::Result<JobInfo> {
        let url = format!("{}/job/{}", self.base_url, job_id);
        let res = self.client.get(&url).send().await?;
        check_response(&res)?;
        let job: JobInfo = res.json().await?;
        Ok(job)
    }

    pub async fn delete_job(&self, job_id: JobId) -> anyhow::Result<()> {
        let url = format!("{}/job/{}", self.base_url, job_id);
        let res = self.client.delete(&url).send().await?;
        check_response(&res)?;
        Ok(())
    }

    pub async fn get_queues(&self) -> anyhow::Result<Vec<String>> {
        let url = format!("{}/queue", self.base_url);
        let res = self.client.get(&url).send().await?;
        check_response(&res)?;
        let queues: Vec<String> = res.json().await?;
        Ok(queues)
    }

    pub async fn fetch_filtered(&self, filter: JobFilter) -> anyhow::Result<Vec<JobInfo>> {
        let queues = if filter.queue.is_empty() {
            self.get_queues().await?
        } else {
            filter.queue.clone()
        };

        let ids = if !filter.tag.is_empty() {
            self.fetch_ids_by_tags(&filter.tag).await?
        } else {
            self.fetch_ids_by_queues_and_status(&queues, &filter.status)
                .await?
        };
        let jobs = self.fetch_jobs_by_ids_and_filter(ids, filter).await?;
        Ok(jobs)
    }

    async fn fetch_ids_by_tags(&self, tags: &[String]) -> anyhow::Result<Vec<JobId>> {
        let ids = join_all(tags.iter().map(|tag| self.fetch_ids_by_tag(tag))).await;
        let ids = ids
            .into_iter()
            .filter_map(|res| res.ok())
            .flatten()
            .collect();
        Ok(ids)
    }

    async fn fetch_ids_by_tag(&self, tag: &str) -> anyhow::Result<Vec<JobId>> {
        let url = format!("{}/tag/{}", self.base_url, tag);
        let res = self.client.get(&url).send().await?;
        check_response(&res)?;
        let res: Vec<JobId> = res.json().await?;
        Ok(res)
    }

    async fn fetch_ids_by_queues_and_status(
        &self,
        queues: &[String],
        status: &[JobStatus],
    ) -> anyhow::Result<Vec<JobId>> {
        let ids = join_all(
            queues
                .iter()
                .map(|queue| self.fetch_ids_by_queue_and_status(queue, status)),
        )
        .await;
        let ids = ids
            .into_iter()
            .filter_map(|ids| ids.ok())
            .flatten()
            .collect();
        Ok(ids)
    }

    async fn fetch_ids_by_queue_and_status(
        &self,
        queue: &str,
        status: &[JobStatus],
    ) -> anyhow::Result<Vec<JobId>> {
        let url = format!("{}/queue/{}/job_ids", self.base_url, queue);
        let res = self.client.get(&url).send().await?;
        check_response(&res)?;
        let res: HashMap<String, Vec<JobId>> = res.json().await?;
        let job_ids = if status.is_empty() {
            res.values().flatten().copied().collect()
        } else {
            res.into_iter()
                .filter_map(|(s, ids)| {
                    let s: JobStatus = serde_json::from_value(serde_json::Value::String(s))
                        .expect("Failed to parse ocypod status");
                    if status.contains(&s) {
                        Some(ids)
                    } else {
                        None
                    }
                })
                .flatten()
                .collect()
        };
        Ok(job_ids)
    }

    async fn fetch_jobs_by_ids_and_filter(
        &self,
        ids: Vec<JobId>,
        filter: JobFilter,
    ) -> anyhow::Result<Vec<JobInfo>> {
        let jobs = join_all(ids.into_iter().map(|id| self.get_job(id))).await;
        let jobs = jobs
            .into_iter()
            .filter_map(|res| res.ok())
            .filter(|job| filter.matches(job))
            .collect();
        Ok(jobs)
    }
}

fn check_response(res: &reqwest::Response) -> anyhow::Result<()> {
    if res.status().is_success() {
        Ok(())
    } else {
        Err(anyhow::anyhow!(res.status().canonical_reason().unwrap()))
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Default)]
pub struct JobFilter {
    #[serde(default)]
    pub queue: Vec<String>,
    #[serde(default)]
    pub status: Vec<JobStatus>,
    #[serde(default)]
    pub tag: Vec<String>,
}

impl JobFilter {
    pub fn all() -> Self {
        Self::default()
    }

    pub fn new(queue: Vec<String>, status: Vec<JobStatus>, tag: Vec<String>) -> Self {
        Self { queue, status, tag }
    }

    pub fn with_queue(mut self, queue: impl ToString) -> Self {
        self.queue.push(queue.to_string());
        self
    }

    pub fn with_status(mut self, status: JobStatus) -> Self {
        self.status.push(status);
        self
    }

    pub fn with_tag(mut self, tag: String) -> Self {
        self.tag.push(tag);
        self
    }

    pub fn matches(&self, job: &JobInfo) -> bool {
        if !self.status.is_empty() && !self.status.contains(&job.status) {
            return false;
        }
        if !self.queue.is_empty() && !self.queue.contains(&job.queue) {
            return false;
        }
        if !self.tag.is_empty() {
            for tag in &self.tag {
                if job.tags.contains(tag) {
                    return true;
                }
            }
            return false;
        }
        true
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
struct JobUpdate {
    pub status: Option<JobStatus>,
    pub output: Option<JobOutput>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    Completed,
    Failed,
    Cancelled,
    // these cannot be set manually
    Queued,
    Running,
    TimedOut,
}

impl JobStatus {
    pub fn pending(&self) -> bool {
        matches!(self, Self::Queued | Self::Running)
    }

    pub fn failed(&self) -> bool {
        matches!(self, Self::Failed | Self::Cancelled | Self::TimedOut)
    }

    pub fn completed(&self) -> bool {
        matches!(self, Self::Completed)
    }
}

#[derive(Default, Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct JobCreate {
    pub input: serde_json::Value,
    pub tags: Vec<String>,
    pub timeout: Option<Duration>,
    pub heartbeat_timeout: Option<Duration>,
    pub expires_after: Option<Duration>,
    pub retries: Option<u32>,
    pub retry_delays: Vec<Duration>,
}

impl JobCreate {
    pub fn with_defaults(input: serde_json::Value) -> Self {
        Self {
            input,
            ..Default::default()
        }
    }
    pub fn with_tags(input: serde_json::Value, tags: Vec<String>) -> Self {
        Self {
            input,
            tags,
            ..Default::default()
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct JobInput {
    pub id: u64,
    pub input: serde_json::Value,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct JobInfo {
    pub id: u64,
    pub queue: String,
    pub status: JobStatus,
    #[serde(default)]
    pub tags: Vec<String>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub ended_at: Option<chrono::DateTime<chrono::Utc>>,
    pub last_heartbeat: Option<chrono::DateTime<chrono::Utc>>,
    pub input: serde_json::Value,
    pub output: Option<JobOutput>,
    pub timeout: Duration,
    pub heartbeat_timeout: Duration,
    pub expires_after: Duration,
    pub retries: u32,
    pub retries_attempted: u32,
    pub retry_delays: Option<Vec<Duration>>,
    pub ended: bool,
}

impl JobInfo {
    pub fn ended_later_than(&self, other: &JobInfo) -> Option<bool> {
        match (self.ended_at, other.ended_at) {
            (Some(_), None) => Some(true),
            (None, Some(_)) => Some(false),
            (Some(me), Some(other)) => Some(me > other),
            (None, None) => None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct JobOutput {
    pub progress: Option<f32>,
    pub error: Option<serde_json::Value>,
    #[serde(default)]
    pub meta: Option<HashMap<String, String>>,
    pub duration: Option<f32>,
}

impl JobOutput {
    pub fn merge(&mut self, other: &JobOutput) {
        if let Some(progress) = other.progress {
            self.progress = Some(progress);
        }
        if let Some(error) = &other.error {
            self.error = Some(error.clone());
        }
        if let Some(meta) = &other.meta {
            if self.meta.is_none() {
                self.meta = Some(HashMap::new());
            }
            let meta_mut = self.meta.as_mut().unwrap();
            for (key, value) in meta {
                meta_mut.insert(key.to_string(), value.to_string());
            }
        }
    }
}

// Models
// #[serde(default)]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Queue {
    pub timeout: Duration,
    pub heartbeat_timeout: Duration,
    pub expires_after: Duration,
    pub retries: u64,
    pub retry_delays: Vec<Duration>,
}

/// Duration to second resolution, thin wrapper around `time::Duration` allowing for custom
/// (de)serialisation.
///
/// Serialised to/from JSON as a human readable time (e.g. "1m", "1day", "1h 22m 58s").
/// Serialised to/from Redis as u64 seconds.
#[derive(Clone, Debug, PartialEq)]
pub struct Duration(pub time::Duration);
impl Serialize for Duration {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&humantime::format_duration(self.0).to_string())
    }
}

impl From<time::Duration> for Duration {
    fn from(dur: time::Duration) -> Self {
        Self(dur)
    }
}

impl<'de> Deserialize<'de> for Duration {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Duration, D::Error> {
        let s: &str = Deserialize::deserialize(deserializer)?;
        humantime::parse_duration(s)
            .map(Duration)
            .map_err(D::Error::custom)
    }
}

impl fmt::Display for Duration {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", humantime::format_duration(self.0))
    }
}

impl From<Duration> for serde_json::Value {
    fn from(duration: Duration) -> Self {
        serde_json::Value::String(duration.to_string())
    }
}
