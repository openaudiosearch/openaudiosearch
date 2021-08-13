use crate::couch::{CouchDB, PutResult};
use oas_common::{types::Post, util};
use oas_common::{Reference, UntypedRecord};
use rss::extension::ExtensionMap;
use rss::Channel;
use std::collections::HashMap;
use std::time::Duration;
use url::{ParseError, Url};

use crate::types::{FeedSettings, Media};
use crate::Record;
pub mod crawlers;
mod error;
pub mod manager;
pub mod mapping;
pub mod ops;

pub use error::{RssError, RssResult};
pub use ops::{Crawler, FetchedFeedPage, Next};

#[derive(Debug, Clone)]
pub struct FeedWatcher {
    url: Url,
    client: reqwest::Client,
    channel: Option<Channel>,
    settings: FeedSettings,
    mapping: HashMap<String, String>,
}

impl FeedWatcher {
    pub fn new(
        url: impl AsRef<str>,
        settings: Option<FeedSettings>,
        mapping: HashMap<String, String>,
    ) -> Result<Self, ParseError> {
        let client = reqwest::Client::new();
        Self::with_client(client, url, settings, mapping)
    }

    pub fn with_client(
        client: reqwest::Client,
        url: impl AsRef<str>,
        settings: Option<FeedSettings>,
        mapping: HashMap<String, String>,
    ) -> Result<Self, ParseError> {
        let url = url.as_ref().parse()?;
        let feed = Self {
            url,
            client,
            channel: None,
            settings: settings.unwrap_or_default(),
            mapping,
        };
        Ok(feed)
    }

    pub fn url(&self) -> &Url {
        &self.url
    }
    pub async fn watch(&mut self, db: CouchDB) -> Result<(), RssError> {
        let duration = Duration::from_secs(self.settings.check_interval);
        let mut interval = tokio::time::interval(duration);
        loop {
            self.load().await?;
            self.save(&db, false).await?;
            interval.tick().await;
        }
    }

    pub async fn save(
        &mut self,
        db: &CouchDB,
        update: bool,
    ) -> Result<(Vec<PutResult>, Vec<UntypedRecord>), RssError> {
        let records = self.to_post_and_media_records()?;
        let put_result = if update {
            db.put_untyped_record_bulk_update(records.clone()).await?
        } else {
            db.put_untyped_record_bulk(records.clone()).await?
        };

        let (success, error): (Vec<_>, Vec<_>) = put_result
            .iter()
            .partition(|r| matches!(r, PutResult::Ok(_)));

        log::debug!(
            "saved posts from feed {} ({} success, {} error)",
            self.url,
            success.len(),
            error.len()
        );
        Ok((put_result, records))
    }

    pub async fn load(&mut self) -> Result<(), RssError> {
        let res = self.client.get(self.url.as_str()).send().await?;
        if !res.status().is_success() {
            return Err(RssError::RemoteHttpError(Box::new(res)));
        }
        let bytes = res.bytes().await?;
        let channel = Channel::read_from(&bytes[..])?;
        self.channel = Some(channel);
        Ok(())
    }

    pub fn to_post_and_media_records(&self) -> Result<Vec<UntypedRecord>, RssError> {
        let posts = self.to_posts()?;
        let mut docs = vec![];
        for mut post in posts.into_iter() {
            let mut refs = post.extract_refs();
            docs.append(&mut refs);
            // TODO: Handle error?
            if let Ok(record) = post.into_untyped_record() {
                docs.push(record);
            }
        }
        Ok(docs)
    }

    pub fn to_posts(&self) -> Result<Vec<Record<Post>>, RssError> {
        if self.channel.is_none() {
            return Err(RssError::NoChannel);
        }
        let channel = self.channel.as_ref().unwrap();
        let mut records = vec![];
        for item in channel.items() {
            let record = item_into_post(&self.mapping, item.clone());
            records.push(record);
        }
        Ok(records)
    }

    pub fn to_medias(&self) -> Result<Vec<Record<Media>>, RssError> {
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

fn resolve_extensions(
    extensions: &rss::extension::ExtensionMap,
    mapping: &HashMap<String, String>,
) -> HashMap<String, String> {
    let result: HashMap<String, String> = mapping
        .iter()
        .filter_map(|(rss_ext_key, target_key)| {
            let mut parts = rss_ext_key.split(":");
            match (parts.next(), parts.next()) {
                (Some(prefix), Some(suffix)) => {
                    let value = extensions
                        .get(prefix)
                        .and_then(|inner_map| inner_map.get(suffix))
                        .and_then(|extension| extension.get(0))
                        .and_then(|extension| extension.value().map(|value| value.to_string()))
                        .map(|value| (target_key.to_string(), value));
                    value
                }
                _ => None,
            }
        })
        .collect();
    result
}

fn item_into_post(mapping: &HashMap<String, String>, item: rss::Item) -> Record<Post> {
    // Create initial post by parsing extension values from the RSS item
    // and deserializing via serde into the Post struct. Further regular
    // values will be set on this struct manually (see below.)
    // TODO: implement mapping management (load mapping, save mapping)
    let extensions: &ExtensionMap = item.extensions();
    let mapped_fields = resolve_extensions(extensions, mapping);
    let mut post = {
        //let mapped_fields  = mapped_fields.into_iter().filter(|(k,_v)| !(k.starts_with("media.")));
        let mapped_fields_json: serde_json::Map<String, serde_json::Value> = mapped_fields
            .clone()
            .into_iter()
            .map(|(k, v)| (k, serde_json::Value::String(v)))
            .filter(|(k, _v)| !(k.starts_with("media.")))
            .collect();
        let post: Result<Post, serde_json::Error> =
            serde_json::from_value(serde_json::Value::Object(mapped_fields_json));
        let post = post.unwrap_or_default();
        post
    };

    // If the RSS item has an enclosure set create a Media record that will be referenced by the post.
    let media = if let Some(enclosure) = item.enclosure {
        let mut mapped_fields_json: serde_json::Map<String, serde_json::Value> = mapped_fields
            .into_iter()
            .filter(|(k, _v)| k.starts_with("media."))
            .map(|(k, v)| {
                let arr: Vec<&str> = k.split(".").collect();
                let v = serde_json::Value::String(v);
                let k = arr[1].into();
                (k, v)
            })
            .collect();
        mapped_fields_json.insert(
            "contentUrl".into(),
            serde_json::Value::String(enclosure.url),
        );
        mapped_fields_json.insert(
            "encodingFormat".into(),
            serde_json::Value::String(enclosure.mime_type),
        );
        let media: Result<Media, serde_json::Error> =
            serde_json::from_value(serde_json::Value::Object(mapped_fields_json));
        let media = media.unwrap_or_default();
        let media =
            Record::from_id_and_value(util::id_from_hashed_string(&media.content_url), media);
        let media_ref = Reference::Resolved(media);
        vec![media_ref]
    } else {
        vec![]
    };

    // Set standard properties from the RSS item on the Post.
    let guid = item.guid.clone();
    post.headline = item.title.clone();
    post.url = item.link.clone();
    post.identifier = guid.as_ref().map(|guid| guid.value().to_string());
    post.media = media;
    if let Some(rfc_2822_date) = item.pub_date {
        if let Ok(date) = chrono::DateTime::parse_from_rfc2822(&rfc_2822_date) {
            post.date_published = Some(date.to_rfc3339());
        }
    }
    if let Some(creator) = item.author {
        post.creator.push(creator.to_string());
    }
    for category in item.categories {
        post.genre.push(category.name);
    }

    // TODO: What to do with items without GUID?
    let guid = guid.unwrap();
    let id = util::id_from_hashed_string(guid.value().to_string());
    Record::from_id_and_value(id, post)
}

fn item_into_record(item: rss::Item) -> Record<Media> {
    let guid = item.guid.clone();
    let mut value = Media {
        ..Default::default()
    };
    if let Some(enclosure) = item.enclosure {
        value.content_url = enclosure.url;
        value.encoding_format = Some(enclosure.mime_type);
    }

    // TODO: What to do with items without GUID?
    let guid = guid.unwrap();
    let id = util::id_from_hashed_string(guid.value().to_string());
    Record::from_id_and_value(id, value)
}
