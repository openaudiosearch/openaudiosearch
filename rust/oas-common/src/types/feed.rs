use crate::mapping::Mappable;
use crate::record::{TypedValue, ValidationError};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use url::Url;

use super::media::MediaTasks;
use super::post::PostTasks;

// pub type Mapping = serde_json::Value;

pub const DEFAULT_CHECK_INTERVAL: u64 = 3600;

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FeedSettings {
    /// Interval to check for feed updates (in seconds)
    pub check_interval: u64,
    /// Try to crawl the feed backwards by increasing an offset query parameter
    pub crawl_backwards: bool,
}

impl Default for FeedSettings {
    fn default() -> Self {
        Self {
            check_interval: DEFAULT_CHECK_INTERVAL,
            crawl_backwards: false,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FeedState {
    pub crawl_finished: bool,
    pub crawl_last_offset: usize,
    // pub last_check: FeedCheckState,
}

// #[derive(Serialize, Deserialize, Debug, Clone, Default, JsonSchema)]
// #[serde(rename_all = "camelCase")]
// pub struct FeedCheckState {
//     timestamp: u32,
//     latest_guid: String,
// }

#[derive(Serialize, Deserialize, Debug, Clone, Default, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Feed {
    pub url: String,
    pub settings: Option<FeedSettings>,
    pub task_defaults: Option<FeedTaskDefaults>,
    // #[serde(default)]
    // pub mapping: Mapping,
    // pub state: Option<FeedState>
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FeedTaskDefaults {
    pub media: Option<MediaTasks>,
    pub post: Option<PostTasks>,
}

impl Feed {
    pub fn with_url(url: String) -> Self {
        Self {
            url,
            ..Default::default()
        }
    }
}

impl TypedValue for Feed {
    const NAME: &'static str = "oas.Feed";

    fn validate(&self) -> Result<(), ValidationError> {
        let _url = Url::parse(&self.url)?;
        Ok(())
    }
}

impl Mappable for Feed {}
