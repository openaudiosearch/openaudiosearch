use anyhow::Context;
use chrono::Utc;
use futures::stream::StreamExt;
use futures_batch::ChunksTimeoutStreamExt;
use oas_common::task::{TaskRunningState, TaskState};
use oas_common::types::{Media, Post};
use oas_common::{Record, RecordMap, Resolver, TypedValue, UntypedRecord};
use serde::{Deserialize, Serialize};
use std::time;

use super::taskdefs;
use super::CeleryManager;
use crate::couch::changes::changes_into_untyped_records;
use crate::couch::{ChangeEvent, CouchDB, CouchResult};
use crate::State;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct TaskProcessState {
    latest_seq: String,
}

impl TypedValue for TaskProcessState {
    const NAME: &'static str = "oas.TaskProcessState";
}

const TASK_STATE_ID: &str = "default";

pub async fn process_changes(state: State, infinite: bool) -> anyhow::Result<()> {
    let db = state.db_manager.record_db();
    let celery = state.tasks;
    let meta_db = state.db_manager.meta_db();

    let latest_seq = get_latest_seq(&meta_db).await;
    log::debug!("start task process at couchdb seq {:?}", latest_seq);
    let mut changes = db.changes(latest_seq.clone());
    changes.set_infinite(infinite);

    let batch_timeout = time::Duration::from_millis(200);
    let batch_max_len = 1000;
    let mut changes = changes.chunks_timeout(batch_max_len, batch_timeout);

    while let Some(batch) = changes.next().await {
        let last_seq = last_seq_of_batch(&batch);
        let batch = changes_into_untyped_records(batch);
        process_batch(&celery, db.clone(), batch)
            .await
            .context("Failed to process changes batch for tasks")?;
        if let Some(next_latest_seq) = last_seq {
            if &next_latest_seq != latest_seq.as_deref().unwrap_or("") {
                log::trace!(
                    "save task process state at couchdb seq: {}",
                    next_latest_seq
                );
                save_latest_seq(&meta_db, next_latest_seq)
                    .await
                    .context("Failed to save task meta state to CouchDB")?;
            }
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

async fn process_post(
    celery: &CeleryManager,
    _db: &CouchDB,
    record: Record<Post>,
) -> anyhow::Result<()> {
    if let Some(tasks) = record.task_states() {
        if let Some(TaskState::Wanted) = &tasks.nlp {
            let opts = serde_json::Value::Null;
            celery
                .send_task(taskdefs::nlp2::new(record.clone(), opts))
                .await
                .context("failed to send task")?;
        }
    }
    Ok(())
}

async fn process_media(
    celery: &CeleryManager,
    db: &CouchDB,
    record: Record<Media>,
) -> anyhow::Result<()> {
    if let Some(tasks) = record.task_states() {
        if let Some(TaskState::Wanted) = &tasks.asr {
            let task_id = celery
                .transcribe_media(&record)
                .await
                .context("failed to send task")?;
            let mut next_record = record.clone();
            let state = TaskRunningState {
                task_id: task_id.to_string(),
                start: Utc::now(),
            };
            next_record.task_states_mut().unwrap().asr = Some(TaskState::Running(state));
            db.put_record(next_record).await?;
        }
        if let Some(TaskState::Wanted) = &tasks.download {
            let opts = serde_json::Value::Null;
            celery
                .send_task(taskdefs::download2::new(record.clone(), opts))
                .await
                .context("failed to send task")?;
        }
    }
    Ok(())
}

pub async fn process_batch(
    celery: &CeleryManager,
    db: CouchDB,
    batch: Vec<UntypedRecord>,
) -> anyhow::Result<()> {
    let mut sorted = RecordMap::from_untyped(batch)?;
    let mut posts = sorted.into_vec::<Post>();
    let medias = sorted.into_vec::<Media>();
    db.resolve_all_refs(&mut posts)
        .await
        .context("failed to resolve refs")?;

    for record in posts.into_iter() {
        let res = process_post(&celery, &db, record).await;
        log_if_error(res);
    }

    for record in medias.into_iter() {
        // log::trace!(
        //     "task process touch media {}, task states: {:?}",
        //     record.id(),
        //     record.task_states()
        // );
        let res = process_media(&celery, &db, record).await;
        log_if_error(res);
    }

    Ok(())
}

fn log_if_error<A>(res: anyhow::Result<A>) {
    match res {
        Ok(_) => {}
        Err(err) => log::error!("{:?}", err),
    }
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
