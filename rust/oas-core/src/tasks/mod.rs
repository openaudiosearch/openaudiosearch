use celery::broker::RedisBroker;
use celery::task::Signature;
use celery::{broker::Broker, prelude::CeleryError, Celery};
use clap::Clap;
use oas_common::{
    types::{Media, Post},
    Record, TypedValue,
};
use serde_json::{json, Value};
use std::fmt;
use std::sync::Arc;

mod taskdefs;

use crate::couch::CouchDB;
use crate::State;

// use self::changes::process_changes;

pub mod changes;

pub const DEFAULT_REDIS_URL: &str = "redis://127.0.0.1:6379/";

pub type CeleryState = Arc<Celery<RedisBroker>>;

#[derive(Debug, Clone)]
pub struct Config {
    redis_url: String,
}

impl Config {
    pub fn from_redis_url_or_default(redis_url: Option<&str>) -> Self {
        let redis_url = redis_url.unwrap_or(DEFAULT_REDIS_URL).to_string();
        Self { redis_url }
    }
}

#[derive(Clap)]
pub struct TaskOpts {
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

#[derive(Clone)]
pub struct CeleryManager {
    config: Config,
    celery: Option<Arc<Celery<RedisBroker>>>,
}

impl fmt::Debug for CeleryManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CeleryManager")
    }
}

impl CeleryManager {
    pub fn with_config(config: Config) -> Self {
        Self {
            config,
            celery: None,
        }
    }

    pub fn with_url<S>(url: Option<S>) -> anyhow::Result<Self>
    where
        S: AsRef<str>,
    {
        let url = url.map(|s| s.as_ref().to_string());
        let config = Config::from_redis_url_or_default(url.as_deref());
        Ok(Self::with_config(config))
    }

    pub async fn init(&mut self) -> Result<(), CeleryError> {
        let celery = create_celery_app(&self.config).await?;
        self.celery = Some(celery);
        Ok(())
    }

    pub fn celery(&self) -> Result<&Arc<Celery<RedisBroker>>, CeleryError> {
        self.celery
            .as_ref()
            .ok_or_else(|| CeleryError::ForcedShutdown)
    }

    pub async fn send_task<T: celery::task::Task>(
        &self,
        task_sig: Signature<T>,
    ) -> Result<celery::task::AsyncResult, CeleryError> {
        let res = self.celery()?.send_task(task_sig).await?;
        log::debug!(
            "created task {} with id {}",
            Signature::<T>::task_name(),
            res.task_id
        );
        Ok(res)
    }

    // pub async fn post_result(&self,

    pub async fn transcribe_media(&self, media: &Record<Media>) -> anyhow::Result<String> {
        let celery = self.celery()?;

        let args = json!({
            "media_url": &media.value.content_url,
            "media_id": media.id()
        });
        let opts = Value::Object(Default::default());
        let res = celery
            .send_task(taskdefs::transcribe::new(args, opts))
            .await?;
        log::debug!(
            "Create asr task for media {} (task id {})",
            media.id(),
            res.task_id
        );
        Ok(res.task_id)
    }

    pub async fn nlp_post(&self, post: &Record<Post>) -> anyhow::Result<String> {
        let celery = self.celery()?;

        let opts = Value::Object(Default::default());
        let res = celery
            .send_task(taskdefs::nlp2::new(post.clone(), opts))
            .await?;
        log::debug!(
            "Create nlp task for post {} (task id {})",
            post.id(),
            res.task_id
        );
        Ok(res.task_id)
    }
}

pub async fn create_celery_app(config: &Config) -> Result<Arc<Celery<RedisBroker>>, CeleryError> {
    let url = &config.redis_url;
    let app = celery::app!(
        broker = RedisBroker { url },
        tasks = [
            taskdefs::transcribe,
            taskdefs::download2,
            taskdefs::nlp2,
        ],
        task_routes = [
            "*" => "celery",
        ],
        prefetch_count = 2,
        heartbeat = Some(10),
    )
    .await?;
    Ok(app)
}

pub async fn load_medias_for_task_opts(
    db: &CouchDB,
    opts: &TaskOpts,
) -> anyhow::Result<Vec<Record<Media>>> {
    let medias: Vec<Record<Media>> = match opts {
        TaskOpts {
            post: None,
            media: Some(id),
            ..
        } => {
            vec![db.get_record::<Media>(&Media::guid(&id)).await?]
        }
        TaskOpts {
            media: None,
            post: Some(id),
            ..
        } => {
            let mut post = db.get_record::<Post>(&Post::guid(&id)).await?;
            post.resolve_refs(db).await?;
            post.value
                .media
                .into_iter()
                .filter_map(|r| r.into_record())
                .collect()
        }
        TaskOpts { missing: true, .. } => db
            .table::<Media>()
            .get_all()
            .await?
            .into_iter()
            .filter_map(|r| match r.value.transcript {
                None => Some(r),
                Some(_) => None,
            })
            .collect(),
        TaskOpts { latest: true, .. } => db.table::<Media>().get_all().await?,
        _ => {
            anyhow::bail!("Invalid or ambigous options")
        }
    };
    Ok(medias)
}

pub async fn run_celery(mut state: State, opts: TaskOpts) -> anyhow::Result<()> {
    state.tasks.init().await?;
    state.db.init().await?;
    let medias = load_medias_for_task_opts(&state.db, &opts).await?;
    if medias.is_empty() {
        anyhow::bail!("No media found")
    }

    println!("Creating tasks for {} medias", medias.len());
    for media in medias {
        match state.tasks.transcribe_media(&media).await {
            Ok(task_id) => println!(
                "created transcribe task for media {}, task id: {}",
                media.id(),
                task_id
            ),
            Err(err) => println!(
                "error creating transcribe task for media {}: {}",
                media.id(),
                err
            ),
        }
    }

    Ok(())
}

pub async fn create_transcribe_task<B>(
    app: &Arc<Celery<B>>,
    media: &Record<Media>,
) -> anyhow::Result<String>
where
    B: Broker + 'static,
{
    let args = json!({
        "media_url": &media.value.content_url,
        "media_id": media.id()
    });
    let opts = Value::Object(Default::default());
    let res = app.send_task(taskdefs::transcribe::new(args, opts)).await?;
    Ok(res.task_id)
}
