use anyhow::Context;
use oas_common::types::{Media, Post};
use oas_common::{Record, RecordMap, Resolver, TypedValue, UntypedRecord};

use crate::couch::durable_changes::ChangesOpts;
use crate::jobs::{typs as job_typs, JobInfo};
use crate::State;

use super::JobManager;

const DURABLE_ID: &str = "core.jobs";

pub async fn process_changes(state: State, infinite: bool) -> anyhow::Result<()> {
    let opts = ChangesOpts {
        infinite,
        ..Default::default()
    };
    let mut changes = state.db_manager.durable_changes(DURABLE_ID, opts).await;
    while let Some(batch) = changes.next().await? {
        process_batch(&state, batch.into_inner())
            .await
            .context("Failed to process changes batch for jobs")?;
    }
    Ok(())
}

pub async fn process_batch(state: &State, batch: Vec<UntypedRecord>) -> anyhow::Result<()> {
    let mut sorted = RecordMap::from_untyped(batch)?;
    let mut posts = sorted.into_vec::<Post>();
    let medias = sorted.into_vec::<Media>();
    state
        .db
        .resolve_all_refs(&mut posts)
        .await
        .context("failed to resolve refs")?;

    for record in posts.into_iter() {
        let res = process_post(state, record).await;
        log_if_error(res);
    }

    for record in medias.into_iter() {
        let res = process_media(state, record).await;
        log_if_error(res);
    }

    Ok(())
}

async fn process_post(state: &State, record: Record<Post>) -> anyhow::Result<()> {
    // eprintln!("process record {}", record.guid());
    let nlp_job = get_job(&state.jobs, &record, job_typs::NLP).await;
    // check all media asr jobs
    let mut asr_jobs = vec![];
    for media in record.value.media.iter() {
        if let Some(record) = media.record() {
            let asr_job = get_job(&state.jobs, record, job_typs::ASR).await;
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

    if needs_nlp {
        let job = job_typs::nlp_job(&record);
        state.jobs.create_job(job).await?;
    }

    Ok(())
}

async fn process_media(state: &State, record: Record<Media>) -> anyhow::Result<()> {
    let needs_asr = match record.value.transcript {
        Some(_) => false,
        None => match record.meta().latest_job(job_typs::ASR) {
            Some(_job_id) => false,
            None => true,
        },
    };

    if needs_asr {
        let job = job_typs::asr_job(&record);
        state.jobs.create_job(job).await?;
    }

    Ok(())
}

async fn get_job<T: TypedValue>(
    jobs: &JobManager,
    record: &Record<T>,
    typ: &str,
) -> Option<JobInfo> {
    let job_id = record.meta().latest_job(typ);
    if let Some(id) = job_id {
        jobs.job(id).await.ok()
    } else {
        None
    }
}

fn log_if_error<A>(res: anyhow::Result<A>) {
    match res {
        Ok(_) => {}
        Err(err) => log::error!("{:?}", err),
    }
}
