use futures::stream::StreamExt;
use futures_batch::ChunksTimeoutStreamExt;
use oas_common::task::TaskState;
use oas_common::types::{Media, Post};
use oas_common::{Record, RecordMap, Resolver, TypedValue, UntypedRecord};
use serde::{Deserialize, Serialize};
use std::time;

use super::taskdefs;
use super::CeleryManager;
use crate::couch::changes::changes_into_untyped_records;
use crate::couch::{ChangeEvent, CouchDB, CouchResult};

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct TaskProcessState {
    latest_seq: String,
}

impl TypedValue for TaskProcessState {
    const NAME: &'static str = "oas.TaskProcessState";
}

const TASK_STATE_ID: &str = "default";

pub async fn process_changes(
    celery: CeleryManager,
    db: CouchDB,
    infinite: bool,
) -> anyhow::Result<()> {
    let latest_seq = get_latest_seq(&db).await;
    let mut changes = db.changes(latest_seq.clone());
    changes.set_infinite(infinite);

    let batch_timeout = time::Duration::from_millis(200);
    let batch_max_len = 1000;
    let mut changes = changes.chunks_timeout(batch_max_len, batch_timeout);

    while let Some(batch) = changes.next().await {
        let last_seq = last_seq_of_batch(&batch);
        let batch = changes_into_untyped_records(batch);
        process_batch(&celery, db.clone(), batch).await?;
        if let Some(latest_seq) = last_seq {
            let latest_seq = latest_seq.to_string();
            save_latest_seq(&db, latest_seq).await?;
        }
    }
    Ok(())
}

fn last_seq_of_batch(batch: &[CouchResult<ChangeEvent>]) -> Option<String> {
    batch.last().and_then(|v| match v {
        Ok(v) => Some(v.seq.to_string()),
        _ => None,
    })
}

pub async fn process_batch(
    celery: &CeleryManager,
    db: CouchDB,
    batch: Vec<UntypedRecord>,
) -> anyhow::Result<()> {
    let mut sorted = RecordMap::from_untyped(batch)?;
    let mut posts = sorted.into_vec::<Post>();
    let medias = sorted.into_vec::<Media>();
    db.resolve_all_refs(&mut posts).await?;

    for record in posts {
        if let Some(tasks) = record.task_states() {
            if let Some(TaskState::Wanted) = &tasks.nlp {
                let opts = serde_json::Value::Null;
                celery
                    .send_task(taskdefs::nlp2::new(record.clone(), opts))
                    .await?;
            }
        }
    }

    for record in medias {
        if let Some(tasks) = record.task_states() {
            if let Some(TaskState::Wanted) = &tasks.asr {
                celery.transcribe_media(&record).await?;
            }
            if let Some(TaskState::Wanted) = &tasks.download {
                let opts = serde_json::Value::Null;
                celery
                    .send_task(taskdefs::download2::new(record.clone(), opts))
                    .await?;
            }
        }
    }

    Ok(())
}

pub async fn get_latest_seq(db: &CouchDB) -> Option<String> {
    db.table::<TaskProcessState>()
        .get(TASK_STATE_ID)
        .await
        .map(|record| record.value.latest_seq)
        .ok()
}

pub async fn save_latest_seq(db: &CouchDB, latest_seq: String) -> anyhow::Result<()> {
    let record = Record::from_id_and_value(TASK_STATE_ID, TaskProcessState { latest_seq });
    db.table::<TaskProcessState>().put(record).await?;
    Ok(())
}

// pub fn into_typ_buckets(
//     records: Vec<UntypedRecord>,
// ) -> HashMap<String, HashMap<String, UntypedRecord>> {
//     let mut map = HashMap::new();
//     for record in records {
//         let entry = map
//             .entry(record.typ().to_string())
//             .or_insert_with(|| HashMap::new());
//         entry.insert(record.typ().to_string(), record);
//     }
//     map
// }
