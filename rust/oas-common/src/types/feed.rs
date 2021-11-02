use crate::jobs::SettingsMap;
use crate::mapping::Mappable;
use crate::record::{TypedValue, ValidationError};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use url::Url;

pub const DEFAULT_CHECK_INTERVAL: u64 = 3600;

#[derive(Serialize, Deserialize, Debug, Clone, Default, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Feed {
    pub url: String,
    pub settings: Option<FeedSettings>,
    #[serde(default)]
    pub media_jobs: SettingsMap,
    #[serde(default)]
    pub post_jobs: SettingsMap,
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

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FeedSettings {
    /// Interval to check for feed updates (in seconds)
    pub check_interval: u64,
    /// Try to crawl the feed backwards by increasing an offset query parameter
    #[serde(default)]
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
