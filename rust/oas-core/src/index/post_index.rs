use oas_common::types::{Media, Post, Transcript};
use oas_common::{Record, Resolver};
use oas_common::{TypedValue, UntypedRecord};
use serde_json::json;
use std::collections::HashSet;
use std::sync::Arc;
use std::time;

use super::elastic::BulkPutResponse;
use super::{Index, IndexError};
use crate::couch::CouchDB;

pub struct PostIndex {
    index: Arc<Index>,
}

impl PostIndex {
    pub fn new(index: Arc<Index>) -> Self {
        Self { index }
    }

    /// Find all posts that reference any of a list of media ids.
    pub async fn find_posts_for_medias(
        &self,
        media_ids: &[&str],
    ) -> Result<Vec<String>, IndexError> {
        let query = json!({
            "query": {
                "nested": {
                    "path": "media",
                    "score_mode": "avg",
                    "query": {
                        "terms": { "media.$meta.id": media_ids }
                    }
                }
            }
        });
        let res = self.index.query_records(query).await?;
        let ids = res.iter().map(|r| r.id().to_string()).collect();
        Ok(ids)
    }

    pub async fn index_post_by_id(
        &self,
        db: &CouchDB,
        id: &str,
    ) -> anyhow::Result<BulkPutResponse> {
        let id = format!("{}_{}", Post::NAME, id);
        let mut post = db.get_record::<Post>(&id).await?;
        post.resolve_refs(&db).await?;
        if let Some(transcript) = generate_transcript_for_post(&post) {
            post.value.transcript = Some(transcript);
        } else {
        }
        let res = self.index.put_typed_records(&[post]).await?;
        Ok(res)
    }

    pub async fn index_changes(
        &self,
        db: &CouchDB,
        changes: &[UntypedRecord],
    ) -> Result<(), anyhow::Error> {
        let now = time::Instant::now();
        let mut posts = vec![];
        let mut medias_with_posts = HashSet::new();
        let mut medias_without_posts = HashSet::new();
        for record in changes {
            match record.typ() {
                Media::NAME => {
                    medias_without_posts.insert(record.id().to_string());
                }
                Post::NAME => {
                    if let Ok(record) = record.clone().into_typed_record::<Post>() {
                        for media in &record.value.media {
                            medias_with_posts.insert(media.id().to_string());
                        }
                        posts.push(record);
                    }
                }
                _ => {}
            }
        }

        let direct_posts_len = posts.len();

        // Collect the posts for all medias that are in the changes batch and are not referenced
        // from posts in this changes batch.
        let medias_without_posts = medias_without_posts.difference(&medias_with_posts);
        let medias_without_posts: Vec<_> = medias_without_posts.map(|s| s.as_str()).collect();
        let missing_post_ids = self
            .find_posts_for_medias(&medias_without_posts[..])
            .await?;
        let missing_post_ids: Vec<&str> = missing_post_ids.iter().map(|s| s.as_str()).collect();
        let missing_posts = db.get_many_records::<Post>(&missing_post_ids[..]).await?;

        // Add the newley fetched posts to the main posts vec.
        posts.extend(missing_posts.into_iter());

        // Resolve all unresolved media references.
        db.resolve_all_refs(posts.as_mut_slice()).await;

        // Build the transcript for a post.
        for post in posts.iter_mut() {
            if let Some(transcript) = generate_transcript_for_post(&post) {
                post.value.transcript = Some(transcript);
            }
        }

        // Index all records.
        self.index.put_typed_records(&posts).await?;

        log::debug!(
            "indexed {} changes in {} ({} post direct updates, {} media updates resulting in {} post updates)", 
            changes.len(),
            humantime::format_duration(now.elapsed()),
            direct_posts_len,
            medias_without_posts.len(),
            missing_post_ids.len()
        );

        Ok(())
    }
}

fn generate_transcript_for_post(post: &Record<Post>) -> Option<String> {
    let mut post_transcript = "".to_string();
    for (i, media_ref) in post.value.media.iter().enumerate() {
        if let Some(media_record) = media_ref.record() {
            if let Some(transcript) = &media_record.value.transcript {
                let media_transcript = generate_transcript_token_string(&transcript, i);
                post_transcript += " ";
                post_transcript += &media_transcript;
            }
        }
    }
    if post_transcript.is_empty() {
        None
    } else {
        Some(post_transcript)
    }
}

fn generate_transcript_token_string(transcript: &Transcript, id: usize) -> String {
    let mut tokens = vec![];
    for part in transcript.parts.iter() {
        let token = format!(
            "{}|{}:{}:{}:{}",
            part.word, part.start, part.end, part.conf, id
        );
        tokens.push(token);
    }
    let merged = tokens.join(" ");
    merged
}

// fn transform_post_for_elastic(post: &Record<Post>) -> serde_json::Result<serde_json::Value> {
//     let mut value = serde_json::to_value(&post)?;
//     let obj = value.as_object_mut().unwrap();
//     obj.insert("transcript".to_string(), build_post_transcript(&post));
//     Ok(value)
// }

// pub struct Part {
//     media_id: usize,
//     start: f32,
//     end: f32,
//     conf: f32,
//     text: String,
// }

// fn build_post_transcript(post: &Record<Post>) -> serde_json::Value {
//     // let medias: Vec<&Record<Media>> = post
//     let transcripts: Vec<serde_json::Value> = post
//         .value
//         .media
//         .iter()
//         .filter_map(|media| media.record())
//         .map(|media| build_media_transcript(media))
//         .flatten()
//         .collect();
//     // let transcripts: Vec<Vec<_>> = medias
//     //     .iter()
//     //     .map(|media| build_media_transcript(media))
//     //     .collect();
//     // let transcripts: Vec<serde_json::Value> = transcripts.into_iter().flatten().collect();
//     serde_json::Value::Array(transcripts)
// }

// fn build_media_transcript(media: &Record<Media>) -> Vec<serde_json::Value> {
//     vec![]
// }
