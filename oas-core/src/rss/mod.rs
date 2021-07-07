use rss::Channel;
use sha2::Digest;
use url::{ParseError, Url};

use crate::types::Media;
use crate::Record;

pub mod crawlers;
mod error;
pub mod ops;

pub use error::{RssError, RssResult};
pub use ops::{Crawler, FetchedFeedPage, Next};

#[derive(Debug, Clone)]
pub struct Feed {
    url: Url,
    client: surf::Client,
    channel: Option<Channel>,
}

impl Feed {
    pub fn new(url: impl AsRef<str>) -> Result<Self, ParseError> {
        let url = url.as_ref().parse()?;
        let feed = Self {
            url,
            client: surf::Client::new(),
            channel: None,
        };
        Ok(feed)
    }

    pub fn with_client(client: surf::Client, url: impl AsRef<str>) -> Result<Self, ParseError> {
        let url = url.as_ref().parse()?;
        let feed = Self {
            url,
            client,
            channel: None,
        };
        Ok(feed)
    }

    pub fn url(&self) -> &Url {
        &self.url
    }

    pub async fn load(&mut self) -> Result<(), RssError> {
        let req = surf::get(&self.url);
        let mut res = self.client.send(req).await?;
        if !res.status().is_success() {
            return Err(RssError::RemoteHttpError(res));
        }
        let bytes = res.body_bytes().await?;
        let channel = Channel::read_from(&bytes[..])?;
        self.channel = Some(channel);
        Ok(())
    }

    pub fn into_medias(&self) -> Result<Vec<Record<Media>>, RssError> {
        if let Some(channel) = &self.channel {
            let mut records = vec![];
            for item in channel.items() {
                let record = item_into_record(item.clone());
                records.push(record);
            }
            Ok(records)
        } else {
            Err(RssError::NoChannel)
        }
    }
}

fn item_into_record(item: rss::Item) -> Record<Media> {
    let guid = item.guid.clone();
    let mut value = Media {
        headline: item.title,
        url: item.link,
        identifier: guid.as_ref().map(|guid| guid.value().to_string()),
        ..Default::default()
    };
    if let Some(enclosure) = item.enclosure {
        value.content_url = Some(enclosure.url);
        value.encoding_format = Some(enclosure.mime_type);
    }

    // TODO: What to do with items without GUID?
    let guid = guid.unwrap();
    let id = string_to_id(guid.value().to_string());
    let record = Record::from_id_and_value(id, value);
    record
}

fn string_to_id(url: String) -> String {
    let mut hasher = sha2::Sha256::new();
    hasher.update(url.as_bytes());
    let result = hasher.finalize();
    let encoded = base32::encode(base32::Alphabet::Crockford, &result[0..16]);
    encoded.to_lowercase()
    // String::from_utf8(encoded).unwrap()
}
