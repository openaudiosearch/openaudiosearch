//! Index manager
//!
//! The index manager maintains a list of Elasticsearch indexes. It also maintains an "oas.meta"
//! index which stores meta information about the indexing state, most importantly the latest
//! CouchDB seq that was indexed.

use crate::couch::{self, CouchDB};
use elasticsearch::Elasticsearch;
use futures::stream::StreamExt;
use futures_batch::ChunksTimeoutStreamExt;
use oas_common::types::{Media, Post};
use oas_common::UntypedRecord;
use oas_common::{Record, TypedValue};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time;

use super::{elastic, Index};

pub const DEFAULT_PREFIX: &str = "oas";
pub const META_INDEX_NAME: &str = "_meta";

pub const DATA_INDEX_NAME: &str = "data";

pub const DOC_ID_INDEX_STATE: &str = "IndexMeta.data";

#[derive(Debug, Clone)]
pub struct IndexManager {
    config: elastic::Config,
    meta: Arc<Meta>,
    client: Arc<Elasticsearch>,
    data_index: Arc<elastic::Index>, // indexes: HashMap<IndexId, elastic::Index>,
}

#[derive(Debug)]
pub struct Meta {
    index: elastic::Index,
}

#[derive(Debug, PartialEq, Clone)]
pub struct InitOpts {
    delete_meta: bool,
    delete_data: bool,
}

#[derive(Serialize, Deserialize)]
pub struct IndexState {
    last_seq: Option<String>,
}

impl Default for InitOpts {
    fn default() -> Self {
        Self {
            delete_meta: false,
            delete_data: false,
        }
    }
}

impl InitOpts {
    pub fn delete_all() -> Self {
        Self {
            delete_meta: true,
            delete_data: true,
        }
    }

    pub fn delete_data() -> Self {
        Self {
            delete_meta: false,
            delete_data: true,
        }
    }
}

impl Meta {
    pub fn with_index(index: elastic::Index) -> Self {
        Self { index }
    }

    pub async fn latest_indexed_seq(&self) -> anyhow::Result<Option<String>> {
        let id = DOC_ID_INDEX_STATE;
        let doc = self.index.get_doc::<IndexState>(id).await?;
        if let Some(index_state) = doc {
            Ok(index_state.last_seq)
        } else {
            Ok(None)
        }
    }

    pub async fn set_latest_indexed_seq(&self, seq: &str) -> anyhow::Result<()> {
        let id = DOC_ID_INDEX_STATE;
        let index_state = IndexState {
            last_seq: Some(seq.to_string()),
        };
        self.index.put_doc(id, &index_state).await?;
        Ok(())
    }
}

impl IndexManager {
    pub fn with_config(config: elastic::Config) -> Result<Self, elasticsearch::Error> {
        let client = elastic::create_client(config.url.clone())?;
        let client = Arc::new(client);

        let prefix = config.prefix.as_deref().unwrap_or(DEFAULT_PREFIX);
        let meta_index_name = format!("{}.{}", prefix, META_INDEX_NAME);
        let meta_index = Index::with_client_and_name(client.clone(), meta_index_name);
        let meta = Meta::with_index(meta_index);

        let data_index_name = format!("{}.{}", prefix, DATA_INDEX_NAME);
        let data_index = Index::with_client_and_name(client.clone(), data_index_name);

        Ok(Self {
            config,
            client,
            data_index: Arc::new(data_index),
            meta: Arc::new(meta),
        })
    }

    pub async fn init(&self, opts: InitOpts) -> anyhow::Result<()> {
        self.meta.index.ensure_index(opts.delete_meta).await?;
        self.data_index.ensure_index(opts.delete_data).await?;
        Ok(())
    }

    pub fn data_index(&self) -> &Arc<elastic::Index> {
        &self.data_index
    }

    pub async fn index_changes(&self, db: &CouchDB, mode: bool) -> anyhow::Result<()> {
        let index = self.data_index();
        let meta = &self.meta;
        index_changes_stream(&db, &meta, &index, mode).await
    }
}

async fn index_changes_stream(
    db: &CouchDB,
    meta: &Arc<Meta>,
    index: &Arc<Index>,
    infinite: bool,
) -> anyhow::Result<()> {
    let latest_seq = meta.latest_indexed_seq().await?;
    let mut changes = db.changes(latest_seq);
    changes.set_infinite(infinite);

    let batch_timeout = time::Duration::from_millis(200);
    let batch_max_len = 1000;

    let mut batched_changes = changes.chunks_timeout(batch_max_len, batch_timeout);

    while let Some(batch) = batched_changes.next().await {
        // Filter out errors for now.
        let batch: Vec<_> = batch.into_iter().filter_map(|e| e.ok()).collect();
        let _len = batch.len();
        let latest_seq = &batch.last().unwrap().seq.to_string();
        index_changes_batch(db, index, batch).await?;
        meta.set_latest_indexed_seq(latest_seq).await?;
    }

    Ok(())
}

pub async fn posts_into_resolved_posts_and_updated_media_batches(
    db: &CouchDB,
    records: Vec<(UntypedRecord, bool)>,
) -> (Vec<Record<Post>>, Vec<UntypedRecord>) {
    let mut post_batch = vec![];
    let mut media_batch = vec![];
    for (record, is_first_rev) in records.into_iter() {
        match record.typ() {
            Media::NAME => {
                if !is_first_rev {
                    media_batch.push(record);
                }
            }
            Post::NAME => {
                let record = record.into_typed_record::<Post>();
                match record {
                    Ok(record) => {
                        post_batch.push(record)
                        // TODO: Resolve in parallel.
                        // let _res = record.resolve_refs(&db).await;
                        // post_batch.push(record);
                    }
                    Err(e) => log::debug!("{}", e),
                }
            }
            _ => {}
        }
    }

    let resolve_posts_fut: Vec<_> = post_batch
        .into_iter()
        .map(|record| record.into_resolve_refs(&db))
        .collect();
    let post_batch: Vec<_> = futures::future::join_all(resolve_posts_fut)
        .await
        .into_iter()
        .filter_map(|r| r.ok())
        .collect();
    (post_batch, media_batch)
}

pub async fn index_changes_batch(
    db: &CouchDB,
    index: &Arc<Index>,
    changes: Vec<couch::ChangeEvent>,
) -> anyhow::Result<()> {
    let start = time::Instant::now();
    let records_and_is_first_rev: Vec<_> = changes
        .into_iter()
        .filter_map(|event| match event.doc {
            None => None,
            Some(doc) => {
                let is_first_rev = doc.is_first_rev().unwrap_or(true);
                match doc.into_untyped_record() {
                    Err(_) => None,
                    Ok(record) => Some((record, is_first_rev)),
                }
            }
        })
        .collect();

    let (post_batch, media_batch) =
        posts_into_resolved_posts_and_updated_media_batches(&db, records_and_is_first_rev).await;
    let _res = index.put_typed_records(&post_batch).await?;
    // TODO: parallelize?
    for media_record in media_batch.iter() {
        index.update_nested_record("media", &media_record).await?;
    }
    log::debug!(
        "indexed {} posts, {} media updates in {}ms",
        post_batch.len(),
        media_batch.len(),
        start.elapsed().as_millis()
    );
    Ok(())
}

// TODO: Maybe rewrite the loop above onto a struct.
// pub struct ChangesIndexer {
//     stream: ChangesStream,
//     index: Arc<elastic::Index>,
//     interval: tokio::time::Interval,
//     batch: Vec<Record<Post>>,
//     max_batch: usize,
//     total: usize,
// }
// pub enum StreamMode {
//     Finite,
//     Infinite,
// }

// impl From<bool> for StreamMode {
//     fn from(infinite: bool) -> Self {
//         match infinite {
//             true => Self::Infinite,
//             false => Self::Finite,
//         }
//     }
// }

// impl Default for StreamMode {
//     fn default() -> Self {
//         Self::Finite
//     }
// }
