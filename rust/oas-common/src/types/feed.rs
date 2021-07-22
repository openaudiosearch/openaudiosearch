use std::time::Duration;

use crate::mapping::Mappable;
use crate::record::TypedValue;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use url::Url;

pub type Mapping = serde_json::Value;

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
            check_interval: 60,
            crawl_backwards: false,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FeedState {
    pub crawl_finished: bool,
    pub crawl_last_offset: usize,
    pub last_check: FeedCheckState,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FeedCheckState {
    timestamp: u32,
    latest_guid: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Feed {
    pub url: String,
    #[serde(default)]
    pub mapping: Mapping,
    pub settings: Option<FeedSettings>,
    pub state: Option<FeedState>,
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

    fn validate(&self) -> Option<bool> {
        let url = Url::parse(&self.url);
        match url {
            Ok(_url) => {
                eprintln!("TRUE");
                return Some(true);
            }
            Err(_e) => {
                eprintln!("FALSE");
                return Some(false);
            }
        }
    }
}

impl Mappable for Feed {}
