use serde::de::{Deserializer, Error};
use serde::ser::Serializer;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::time;

pub use super::JobId;
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

    pub async fn update_job(
        &self,
        job_id: JobId,
        status: Option<JobStatus>,
        output: Option<serde_json::Value>,
    ) -> anyhow::Result<()> {
        let current = self.get_job(job_id).await?;
        let output = if let Some(output) = output {
            if let Some(current_output) = current.output {
                let mut next_output = current_output.clone();
                json_patch::merge(&mut next_output, &current_output);
                Some(next_output)
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

    pub async fn next_job(&self, queue: &str) -> anyhow::Result<JobInput> {
        let url = format!("{}/queue/{}/job", self.base_url, queue);
        let res = self.client.get(&url).send().await?;
        check_response(&res)?;
        let body: JobInput = res.json().await?;
        Ok(body)
    }

    pub async fn get_queues(&self) -> anyhow::Result<Vec<String>> {
        let url = format!("{}/queue", self.base_url);
        let res = self.client.get(&url).send().await?;
        check_response(&res)?;
        let queues: Vec<String> = res.json().await?;
        Ok(queues)
    }
}

fn check_response(res: &reqwest::Response) -> anyhow::Result<()> {
    if res.status().is_success() {
        Ok(())
    } else {
        Err(anyhow::anyhow!(res.status().canonical_reason().unwrap()))
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
struct JobUpdate {
    pub status: Option<JobStatus>,
    pub output: Option<serde_json::Value>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum JobStatus {
    Completed,
    Failed,
    Canceled,
    // these cannot be set manually
    Queued,
    Running,
    TimedOut,
}

impl JobStatus {
    pub fn pending(&self) -> bool {
        match self {
            Self::Queued | Self::Running => true,
            _ => false,
        }
    }

    pub fn failed(&self) -> bool {
        match self {
            Self::Failed | Self::Canceled | Self::TimedOut => true,
            _ => false,
        }
    }

    pub fn completed(&self) -> bool {
        match self {
            Self::Completed => true,
            _ => false,
        }
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
    pub output: Option<serde_json::Value>,
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
