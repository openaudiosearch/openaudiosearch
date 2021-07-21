use crate::couch::CouchDB;
use oas_common::{types::Post, util};
use rss::Channel;
use std::time::Duration;
use url::{ParseError, Url};

use crate::types::Media;
use crate::{Record, Reference};

pub mod crawlers;
mod error;
pub mod manager;
pub mod ops;

pub use error::{RssError, RssResult};
pub use ops::{Crawler, FetchedFeedPage, Next};

#[derive(Debug, Clone)]
pub struct FeedWatcher {
    url: Url,
    client: surf::Client,
    channel: Option<Channel>,
}

impl FeedWatcher {
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
    pub async fn watch(&mut self, db: CouchDB) -> Result<(), RssError> {
        let mut interval = tokio::time::interval(Duration::new(5, 0));
        loop {
            interval.tick().await;
            self.load().await?;
            let records = self.into_posts()?;
            db.put_record_bulk(records).await?;
        }
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

    pub fn into_posts(&self) -> Result<Vec<Record<Post>>, RssError> {
        if self.channel.is_none() {
            return Err(RssError::NoChannel);
        }
        let channel = self.channel.as_ref().unwrap();
        let mut records = vec![];
        for item in channel.items() {
            let record = item_into_post(item.clone());
            records.push(record);
        }
        Ok(records)
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

fn item_into_post(item: rss::Item) -> Record<Post> {
    let media = if let Some(enclosure) = item.enclosure {
        let media = Media {
            content_url: enclosure.url,
            encoding_format: Some(enclosure.mime_type),
            ..Default::default()
        };
        let media =
            Record::from_id_and_value(util::id_from_hashed_string(&media.content_url), media);
        let media_ref = Reference::Resolved(media);
        vec![media_ref]
    } else {
        vec![]
    };

    let guid = item.guid.clone();
    let value = Post {
        headline: item.title,
        url: item.link,
        identifier: guid.as_ref().map(|guid| guid.value().to_string()),
        media,
        ..Default::default()
    };

    // TODO: What to do with items without GUID?
    let guid = guid.unwrap();
    let id = util::id_from_hashed_string(guid.value().to_string());
    let record = Record::from_id_and_value(id, value);
    record
}

fn item_into_record(item: rss::Item) -> Record<Media> {
    let guid = item.guid.clone();
    let mut value = Media {
        // headline: item.title,
        // url: item.link,
        // identifier: guid.as_ref().map(|guid| guid.value().to_string()),
        ..Default::default()
    };
    if let Some(enclosure) = item.enclosure {
        value.content_url = enclosure.url;
        value.encoding_format = Some(enclosure.mime_type);
    }

    // TODO: What to do with items without GUID?
    let guid = guid.unwrap();
    let id = util::id_from_hashed_string(guid.value().to_string());
    let record = Record::from_id_and_value(id, value);
    record
}
