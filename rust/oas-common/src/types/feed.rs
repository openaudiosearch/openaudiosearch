use crate::mapping::Mappable;
use crate::record::TypedValue;
use serde::{Deserialize, Serialize};

pub type Mapping = serde_json::Value;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct FeedSettings {
    check_interval: f32,
    crawl_backwards: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct FeedState {
    crawl_finished: bool,
    crawl_last_offset: usize,
    last_check: FeedCheckState,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct FeedCheckState {
    timestamp: u32,
    latest_guid: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Feed {
    pub url: String,
    #[serde(default)]
    pub mapping: Mapping,
    pub settings: Option<FeedSettings>,
    pub state: Option<FeedState>,
}

impl TypedValue for Feed {
    const NAME: &'static str = "oas.Feed";
}

impl Mappable for Feed {}
