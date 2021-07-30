use celery::broker::RedisBroker;
use celery::{broker::Broker, prelude::CeleryError, Celery};
use clap::Clap;
use oas_common::{
    types::{Media, Post},
    Record, TypedValue,
};
use serde_json::{json, Value};
use std::fmt;
use std::sync::Arc;

use crate::couch::CouchDB;
use crate::State;

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

mod celery_tasks {
    #![allow(unused)]

    use celery::task::TaskResult;
    use serde_json::Value;

    #[celery::task()]
    pub fn transcribe(args: Value, opts: Value) -> TaskResult<Value> {
        Ok(Value::Null)
    }
}

#[derive(Clap)]
pub struct TaskOpts {
    /// Media ID to enqueue
    #[clap(short, long)]
    media_id: Option<String>,
    /// Post ID to enqueue
    #[clap(short, long)]
    post_id: Option<String>,
    /// Add latest media file
    #[clap(short, long)]
    latest: bool,
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

    pub async fn init(&mut self) -> Result<(), CeleryError> {
        let celery = create_celery_app(&self.config).await?;
        self.celery = Some(celery);
        Ok(())
    }

    pub async fn transcribe_media(&self, media: &Record<Media>) -> anyhow::Result<String> {
        if self.celery.is_none() {
            anyhow::bail!("CeleryManager is not initialized");
        }
        create_transcribe_task(self.celery.as_ref().unwrap(), media).await
    }
}

pub async fn create_celery_app(config: &Config) -> Result<Arc<Celery<RedisBroker>>, CeleryError> {
    let url = &config.redis_url;
    let app = celery::app!(
        broker = RedisBroker { url },
        tasks = [
            celery_tasks::transcribe
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
            post_id: None,
            media_id: Some(id),
            ..
        } => {
            vec![db.get_record::<Media>(&Media::guid(&id)).await?]
        }
        TaskOpts {
            media_id: None,
            post_id: Some(id),
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
        TaskOpts { latest: true, .. } => db
            .get_all_with_prefix(Media::NAME)
            .await?
            .into_typed_records()
            .into_iter()
            .filter_map(|r| r.ok())
            .collect(),
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
    let res = app
        .send_task(celery_tasks::transcribe::new(args, opts))
        .await?;
    Ok(res.task_id)
}
