use async_trait::async_trait;
use celery::broker::RedisBroker;
use celery::task::{TaskResult, TaskResultExt};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Engine {
    Vosk,
    Pytorch,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AsrArgs {
    pub media_file: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AsrOpts {
    pub engine: Engine,
    pub language: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct AsrResult {
    pub text: String,
    pub parts: Vec<serde_json::Value>,
}

// #[celery::task(max_retries = 2)]
// async fn download(media_url: String) {
//     eprintln!("DOWNLOAD: {}", media_url);
// }

#[celery::task()]
pub fn asr(args: AsrArgs, opts: AsrOpts) -> TaskResult<AsrResult> {
    Ok(Default::default())
}

#[celery::task(max_retries = 2)]
async fn add(a: i32, b: i32) -> TaskResult<i32> {
    eprintln!("add: {} + {}", a, b);
    Ok(a + b)
}

pub async fn run_celery() -> anyhow::Result<()> {
    let app = celery::app!(
        broker = RedisBroker { std::env::var("REDIS_ADDR").unwrap_or_else(|_| "redis://127.0.0.1:6379/".into()) },
        tasks = [
            add
        ],
        task_routes = [
            "*" => "celery",
        ],
        prefetch_count = 2,
        heartbeat = Some(10),
    ).await?;
    // let url = "https://dl.arso.xyz/bela1.mp3".to_string();
    let async_result = app.send_task(add::new(7, 8)).await?;
    eprintln!("task id: {}", async_result.task_id);
    let media_file = "/home/bit/Code/oas/open-audio-search/data/oas/f6abce04-b3c3-44bb-a45f-d54905d6e235/processed.wav";
    let asr_args = AsrArgs {
        media_file: media_file.to_string(),
    };
    let asr_opts = AsrOpts {
        engine: Engine::Vosk,
        language: Some("de".into()),
    };
    let async_result = app.send_task(asr::new(asr_args, asr_opts)).await?;
    eprintln!("task id: {}", async_result.task_id);
    Ok(())
}
