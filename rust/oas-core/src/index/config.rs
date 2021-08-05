use clap::Clap;
use url::Url;

pub const DEFAULT_PREFIX: &str = "oas";

/// ElasticSearch config.
#[derive(Clap, Debug, Clone)]
pub struct Config {
    /// Elasticsearch server URL
    #[clap(long, env = "ELASTICSEARCH_URL")]
    pub url: Option<String>,

    // Elasticsearch index
    // #[clap(long, env = "ELASTICSEARCH_INDEX")]
    // pub index: String,
    /// Elasticsearch index prefix
    #[clap(long, env = "ELASTICSEARCH_PREFIX")]
    pub prefix: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            url: None,
            prefix: None,
        }
    }
}

impl Config {
    /// Creates a new config with server URL and index name.
    pub fn new(url: Option<String>) -> Self {
        Self {
            url,
            // index,
            prefix: None,
        }
    }

    pub fn from_url_or_default(url: Option<&str>) -> anyhow::Result<Self> {
        if let Some(url) = url {
            Self::from_url(&url)
        } else {
            Ok(Self::default())
        }
    }

    pub fn from_url(url: &str) -> anyhow::Result<Self> {
        let mut url: Url = url.parse()?;
        let first_segment = url
            .path_segments()
            .map(|mut segments| segments.nth(0).map(|s| s.to_string()))
            .flatten();
        let prefix = if let Some(first_segment) = first_segment {
            first_segment.to_string()
        } else {
            DEFAULT_PREFIX.to_string()
        };
        url.set_path("");
        Ok(Self {
            url: Some(url.to_string()),
            prefix: Some(prefix),
        })
    }

    /// Creates config with index name and default values.
    pub fn with_default_url(prefix: String) -> Self {
        Self {
            url: None,
            prefix: Some(prefix),
        }
    }
}
