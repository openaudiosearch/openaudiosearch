use clap::Parser;
use oas_common::types::{Media, Post};
use oas_common::{Record, TypedValue};

use crate::couch::CouchDB;
use crate::State;

#[derive(Parser, Debug)]
pub struct JobOpts {
    /// Create job
    #[clap(subcommand)]
    command: JobCommand,
}

#[derive(Parser, Debug)]
pub enum JobCommand {
    Create(CreateOpts),
}

#[derive(Parser, Debug)]
pub struct CreateOpts {
    #[clap(subcommand)]
    job: CreateJob,
}

#[derive(Parser, Debug)]
pub enum CreateJob {
    /// NLP
    Nlp(NlpOpts),
    /// ASR
    Asr(AsrOpts),
}

#[derive(Parser, Debug)]
pub struct NlpOpts {
    /// Post ID to enqueue
    #[clap(long)]
    post: String,
}

#[derive(Parser, Debug)]
pub struct AsrOpts {
    /// Media ID to enqueue
    #[clap(long)]
    media: Option<String>,
    /// Post ID to enqueue
    #[clap(long)]
    post: Option<String>,
    /// Add latest media file
    #[clap(long)]
    latest: bool,
    /// Add all medias that don't have a transcript yet.
    #[clap(long)]
    missing: bool,
}

pub async fn main(state: State, opts: JobOpts) -> anyhow::Result<()> {
    state.db.init().await?;
    match opts.command {
        JobCommand::Create(create) => match create.job {
            CreateJob::Nlp(opts) => run_nlp(state, opts).await,
            CreateJob::Asr(opts) => run_asr(state, opts).await,
        },
    }
}

async fn run_nlp(state: State, opts: NlpOpts) -> anyhow::Result<()> {
    let post = state.db.table::<Post>().get(&opts.post).await?;
    let job = crate::jobs::typs::nlp_job(&post, None);
    state.jobs.create_job(job).await?;
    Ok(())
}

async fn run_asr(state: State, opts: AsrOpts) -> anyhow::Result<()> {
    let medias = load_medias_with_opts(&state.db, &opts).await?;
    for media in medias {
        let job = crate::jobs::typs::asr_job(&media, None);
        state.jobs.create_job(job).await?;
    }
    Ok(())
}

pub async fn load_medias_with_opts(
    db: &CouchDB,
    opts: &AsrOpts,
) -> anyhow::Result<Vec<Record<Media>>> {
    let medias: Vec<Record<Media>> = match opts {
        AsrOpts {
            post: None,
            media: Some(id),
            ..
        } => {
            vec![db.get_record::<Media>(&Media::guid(id)).await?]
        }
        AsrOpts {
            media: None,
            post: Some(id),
            ..
        } => {
            let mut post = db.get_record::<Post>(&Post::guid(id)).await?;
            post.resolve_refs(db).await?;
            post.value
                .media
                .into_iter()
                .filter_map(|r| r.into_record())
                .collect()
        }
        AsrOpts { missing: true, .. } => db
            .table::<Media>()
            .get_all()
            .await?
            .into_iter()
            .filter(|r| r.value.transcript.is_none())
            .collect(),
        AsrOpts { latest: true, .. } => db.table::<Media>().get_all().await?,
        _ => {
            anyhow::bail!("Invalid or ambigous options")
        }
    };
    Ok(medias)
}
