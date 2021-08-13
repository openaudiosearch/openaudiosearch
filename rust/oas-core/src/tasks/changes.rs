use futures::stream::StreamExt;
use oas_common::task::TaskState;
use oas_common::types::{Media, Post};
use oas_common::{Record, TypedValue, UntypedRecord};
use serde::{Deserialize, Serialize};

use super::taskdefs;
use super::CeleryManager;
use crate::couch::CouchDB;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct TaskProcessState {
    latest_seq: String,
}

impl TypedValue for TaskProcessState {
    const NAME: &'static str = "oas.TaskProcessState";
}

const TASK_STATE_ID: &str = "default";

// fn last_seq_from_batch(batch: &[Record<
pub async fn process_changes(
    celery: CeleryManager,
    db: CouchDB,
    infinite: bool,
) -> anyhow::Result<()> {
    let latest_seq = get_latest_seq(&db).await;
    let mut changes = db.changes(latest_seq);
    changes.set_infinite(infinite);
    let mut changes = changes.batched_untyped_records();
    while let Some(batch) = changes.next().await {
        for record in batch {
            match record.typ() {
                Post::NAME => {
                    if let Ok(record) = record.into_typed::<Post>() {
                        if let Some(tasks) = record.task_states() {
                            if let Some(TaskState::Wanted) = &tasks.nlp {
                                let opts = serde_json::Value::Null;
                                celery
                                    .send_task(taskdefs::nlp2::new(record.clone(), opts))
                                    .await?;
                            }
                        }
                    }
                }
                Media::NAME => {
                    if let Ok(record) = record.into_typed::<Media>() {
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
                }
                _ => {}
            }
        }

        if let Some(seq) = changes.last_seq() {
            save_latest_seq(&db, seq.to_string()).await?;
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
