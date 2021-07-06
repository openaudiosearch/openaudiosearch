use crate::mapping::Mappable;
use crate::record::TypedValue;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Feed {
    url: String,
}

impl TypedValue for Feed {
    const NAME: &'static str = "oas.Feed";
}

impl Mappable for Feed {}
