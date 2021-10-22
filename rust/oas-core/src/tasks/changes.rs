use anyhow::Context;
use futures::stream::StreamExt;
use futures_batch::ChunksTimeoutStreamExt;
use oas_common::types::{Media, Post};
use oas_common::{Record, RecordMap, Resolver, TypedValue, UntypedRecord};
use serde::{Deserialize, Serialize};
use std::time;

use crate::couch::changes::changes_into_untyped_records;
use crate::couch::{ChangeEvent, CouchDB, CouchResult};
use crate::jobs::{typs as job_typs, JobInfo};
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
    let meta_db = state.db_manager.meta_db();

    let latest_seq = get_latest_seq(&meta_db).await;
    log::debug!("start task process at couchdb seq {:?}", latest_seq);
    let mut changes = db.changes(latest_seq.clone());
    changes.set_infinite(infinite);

    let batch_timeout = time::Duration::from_millis(200);
    let batch_max_len = 1000;
    let mut changes = changes.chunks_timeout(batch_max_len, batch_timeout);

    while let Some(batch) = changes.next().await {
        // eprintln!("got batch with len {}", batch.len());
        let last_seq = last_seq_of_batch(&batch);
        let batch = changes_into_untyped_records(batch);
        process_batch(&state, batch)
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

pub async fn process_batch(state: &State, batch: Vec<UntypedRecord>) -> anyhow::Result<()> {
    // eprintln!("start with {}", batch.len());
    let mut sorted = RecordMap::from_untyped(batch)?;
    let mut posts = sorted.into_vec::<Post>();
    let medias = sorted.into_vec::<Media>();
    state
        .db
        .resolve_all_refs(&mut posts)
        .await
        .context("failed to resolve refs")?;

    for record in posts.into_iter() {
        let res = process_post(&state, record).await;
        log_if_error(res);
    }

    for record in medias.into_iter() {
        let res = process_media(&state, record).await;
        log_if_error(res);
    }

    Ok(())
}

async fn get_job<T: TypedValue>(state: &State, record: &Record<T>, typ: &str) -> Option<JobInfo> {
    let job_id = record.meta().latest_job(typ);
    if let Some(id) = job_id {
        state.jobs.get_job(id).await.ok()
    } else {
        None
    }
}

async fn process_post(state: &State, record: Record<Post>) -> anyhow::Result<()> {
    // eprintln!("process record {}", record.guid());
    let nlp_job = get_job(&state, &record, job_typs::NLP).await;
    // check all media asr jobs
    let mut asr_jobs = vec![];
    for media in record.value.media.iter() {
        if let Some(record) = media.record() {
            let asr_job = get_job(&state, &record, job_typs::ASR).await;
            asr_jobs.push(asr_job);
        }
    }

    // logic is: if there's any asr job that completed later than the latest nlp job a new nlp job
    // should be created.
    let needs_nlp = match nlp_job {
        None => true,
        Some(nlp_job) => match nlp_job.status.pending() {
            true => false,
            false => {
                let mut needs_nlp = false;
                for asr_job in asr_jobs {
                    needs_nlp |= match asr_job {
                        None => false,
                        Some(asr_job) => match asr_job.ended_later_than(&nlp_job) {
                            Some(true) => true,
                            Some(false) | None => false,
                        },
                    };
                }
                needs_nlp
            }
        },
    };

    // eprintln!(" -- record {} needs nlp? {}", record.guid(), needs_nlp);

    if needs_nlp {
        let job = job_typs::nlp_job(&record);
        state.jobs.create_job(job).await?;
    }

    Ok(())
}

async fn process_media(state: &State, record: Record<Media>) -> anyhow::Result<()> {
    // eprintln!("process record {}", record.guid());
    let needs_asr = match record.value.transcript {
        Some(_) => false,
        None => match record.meta().latest_job(job_typs::ASR) {
            Some(_job_id) => false,
            None => true,
        },
    };

    // eprintln!(" -- record {} needs asr? {}", record.guid(), needs_asr);

    if needs_asr {
        let job = job_typs::asr_job(&record);
        state.jobs.create_job(job).await?;
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
