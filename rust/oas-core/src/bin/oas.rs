use anyhow::Context;
use clap::Parser;
use futures::stream::StreamExt;
use oas_common::types::Media;
use oas_common::Record;
use oas_core::rss::manager::FeedManagerOpts;
use oas_core::server::{run_server, ServerOpts};
use oas_core::types::Post;
use oas_core::util::debug_print_records;
use oas_core::{couch, index, jobs, rss};
use oas_core::{Runtime, State};
use std::env;
use std::time;

pub const VAR_LOG: &str = "RUST_LOG";
pub const VAR_FRONTEND_PROXY: &str = "FRONTEND_PROXY";
pub const DEFAULT_FRONTEND_PROXY: &str = "http://localhost:4000";

#[derive(Parser, Debug)]
struct Args {
    #[clap(subcommand)]
    pub command: Command,

    /// Elasticsearch URL
    #[clap(long, env = "ELASTICSEARCH_URL")]
    pub elasticsearch_url: Option<String>,

    /// CouchDB URL
    #[clap(long, env = "COUCHDB_URL")]
    pub couchdb_url: Option<String>,

    /// Redis URL
    #[clap(long, env = "OCYPOD_URL")]
    pub ocypod_url: Option<String>,

    /// Bind HTTP server to host
    #[clap(long, env = "HTTP_HOST")]
    pub http_host: Option<String>,

    /// Bind HTTP server to port
    #[clap(long, env = "HTTP_PORT")]
    pub http_port: Option<u16>,

    /// Path to mapping file
    #[clap(long, env = "MAPPING_FILE")]
    pub mapping_file: Option<String>,

    /// Enable developer mode defaults
    #[clap(long)]
    pub dev: bool,
}

#[derive(Parser, Debug)]
enum Command {
    /// Watch and show print changes from the CouchDB feed
    Watch(WatchOpts),
    /// Print all docs from CouchDB
    List(ListOpts),
    Debug,
    /// Run the indexing pipeline
    Index(IndexOpts),
    /// Search for records
    Search(SearchOpts),
    /// Fetch a RSS feed
    Feed(FeedCommands),
    /// Job commands
    Job(jobs::bin::JobOpts),
    /// Run the HTTP API server
    Server(ServerOpts),
    /// Delete all databases and indexes (dangerous!)
    Nuke,
    /// Run all services
    Run,
}

#[derive(Parser, Debug)]
struct FeedCommands {
    /// Subcommand
    #[clap(subcommand)]
    command: FeedCommand,
}

#[derive(Parser, Debug)]
struct RefetchOpts {
    /// Feed ID or URL
    id_or_url: String,
}

#[derive(Parser, Debug)]
enum FeedCommand {
    /// Fetch a feed by URL.
    Fetch(rss::ops::FetchOpts),
    /// Fetch and crawl a feed by URL (increasing offset param).
    Crawl(rss::ops::CrawlOpts),
    /// Watch on CouchDB changes stream for new feeds
    Watch(FeedManagerOpts),
    /// Refetch a feed and update all records
    Refetch(RefetchOpts),
}

#[derive(Parser, Debug)]
struct IndexOpts {
    /// Run forever in daemon mode
    #[clap(short, long)]
    daemon: bool,

    /// Delete and recreate the index
    #[clap(long)]
    recreate: bool,

    /// Re-index a single post by id.
    #[clap(long)]
    post_id: Option<String>,
}

impl IndexOpts {
    pub fn run_forever() -> Self {
        Self {
            daemon: true,
            recreate: false,
            post_id: None,
        }
    }
}

#[derive(Parser, Debug)]
struct WatchOpts {
    /// Rev to start the watch stream at.
    since: Option<String>,
}

#[derive(Parser, Debug)]
struct SearchOpts {
    /// Search query
    query: String,
    /// Print results as JSON
    #[clap(short, long)]
    json: bool,
}

#[derive(Parser, Debug)]
struct ListOpts {
    /// Type ("post", "media", "feed").
    typ: String,
    /// Only display the number of items.
    #[clap(long)]
    count: bool, // Output as JSON
                 // #[clap(long)]
                 // json: bool
}

fn setup(args: &Args) -> anyhow::Result<()> {
    use tracing_subscriber::EnvFilter;
    if args.dev {
        if env::var(VAR_LOG).is_err() {
            env::set_var(VAR_LOG, "oas=debug")
        }
        if env::var(VAR_FRONTEND_PROXY).is_err() {
            env::set_var(VAR_FRONTEND_PROXY, DEFAULT_FRONTEND_PROXY);
        }
    }
    if env::var(VAR_LOG).is_err() {
        env::set_var(VAR_LOG, "oas=info")
    }

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .try_init()
        .map_err(|e| anyhow::anyhow!(e))?;

    // tracing::debug!(log_level = env::var("RUST_LOG").unwrap(), "start");
    Ok(())
}

fn state_from_args(args: &Args) -> anyhow::Result<State> {
    let db_manager = couch::CouchManager::with_url(args.couchdb_url.as_deref())?;
    let db = db_manager.record_db().clone();
    let index_manager = index::IndexManager::with_url(args.elasticsearch_url.as_deref())?;
    let feed_manager_opts = FeedManagerOpts {
        mapping_file: args.mapping_file.clone(),
    };
    let feed_manager = rss::FeedManager::new(feed_manager_opts);

    let ocypod_url = args
        .ocypod_url
        .as_deref()
        .unwrap_or(jobs::DEFAULT_OCYPOD_URL);
    let job_manager = jobs::JobManager::new(db_manager.record_db().clone(), ocypod_url);

    let state = State::new(db_manager, db, index_manager, feed_manager, job_manager);
    Ok(state)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    setup(&args)?;

    let state = state_from_args(&args)?;

    let now = time::Instant::now();
    let result = match args.command {
        Command::Watch(opts) => run_watch(state, opts).await,
        Command::List(opts) => run_list(state, opts).await,
        Command::Debug => run_debug(state).await,
        Command::Index(opts) => run_index(state, opts).await,
        Command::Search(opts) => run_search(state, opts).await,
        Command::Feed(opts) => run_feed(state, opts.command).await,
        Command::Job(opts) => jobs::bin::main(state, opts).await,
        Command::Server(opts) => run_server(state, opts).await,
        Command::Run => run_all(state, args).await,
        Command::Nuke => run_nuke(state, args).await,
    };
    log::debug!("command took {}", humantime::format_duration(now.elapsed()));
    result
}

async fn run_nuke(state: State, _args: Args) -> anyhow::Result<()> {
    use dialoguer::Input;

    let prompt = "Will DELETE and recreate ALL used CouchDB databases and ElasticSearch indexes. Type \"nuke!\" to continue".to_string();
    println!("{}", prompt);
    let input = Input::<String>::new().interact_text()?;
    if &input == "nuke!" {
        println!("Deleting CouchDB");
        let _ = state.db_manager.destroy_and_init().await;
        println!("Deleting Elasticsearch");
        let _ = state.index_manager.destroy_and_init().await;
    } else {
        println!("exit");
    }
    Ok(())
}

async fn run_all(mut state: State, args: Args) -> anyhow::Result<()> {
    let server_opts = ServerOpts {
        host: args.http_host,
        port: args.http_port,
    };

    state.init_all().await?;

    // A simple abstraction to run tasks and log their results in case of errors.
    let mut runtime = Runtime::new();
    runtime.spawn("server", run_server(state.clone(), server_opts));
    runtime.spawn("index", run_index(state.clone(), IndexOpts::run_forever()));
    // Spawn task watcher.
    runtime.spawn("jobs", jobs::changes::process_changes(state.clone(), true));
    runtime.spawn(
        "feed_watcher",
        state.feed_manager.clone().run_watch(state.db.clone()), // run_feed(state, FeedCommand::Watch(feed_manager_opts)),
    );
    // This calls std::process::exit() on ctrl_c signal.
    // TODO: We might need cancel signals into the tasks for some tasks.
    runtime.spawn("exit", run_exit());

    // This runs until all tasks are finished, i.e. forever.
    runtime.run().await;

    Ok(())
}

async fn run_exit() -> anyhow::Result<()> {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to listen for cancel event");
    std::process::exit(0)
}

async fn run_list(state: State, opts: ListOpts) -> anyhow::Result<()> {
    let db = state.db;
    let records = match opts.typ.as_str() {
        "media" => {
            let records = db.table::<Media>().get_all().await?;
            records
                .into_iter()
                .map(serde_json::to_value)
                .collect::<Vec<_>>()
        }
        "post" => {
            let records = db.table::<Media>().get_all().await?;
            records
                .into_iter()
                .map(serde_json::to_value)
                .collect::<Vec<_>>()
        }
        "feed" => {
            let records = db.table::<Media>().get_all().await?;
            records
                .into_iter()
                .map(serde_json::to_value)
                .collect::<Vec<_>>()
        }
        _ => vec![],
    };
    if opts.count {
        println!("{}", records.len());
        return Ok(());
    }
    for record in records {
        println!("{}", serde_json::to_string(&record.unwrap()).unwrap());
    }
    // log::info!("total {}", len);
    Ok(())
}

async fn run_debug(state: State) -> anyhow::Result<()> {
    let config = crate::jobs::config::JobConfig::load().await;
    let config = config?;
    let args = serde_json::json!({ "media_id": "tbvj6wf1m3ez5051n2dpkh90p8" });
    let argfuns = crate::jobs::argfuns::ArgFunctions::with_defaults();
    let argfun_ctx = crate::jobs::argfuns::ArgFunContext {
        db: state.db.clone(),
    };
    let on_complete_asr = config.on_complete.get("asr").unwrap().get(1).unwrap();
    eprintln!("prev job args {}", args);
    let res = on_complete_asr
        .template_to_args(&args, &argfuns, argfun_ctx)
        .await;
    eprintln!("next job args {:?}", res);
    // eprintln!("OAS debug -- nothing here");
    Ok(())
}

async fn run_watch(state: State, opts: WatchOpts) -> anyhow::Result<()> {
    let db = state.db;
    let mut stream = db.changes(opts.since);
    stream.set_infinite(true);
    while let Some(event) = stream.next().await {
        let event = event?;
        if let Some(doc) = event.doc {
            let record = doc.into_typed_record::<Media>();
            eprintln!("record: {:#?}", record);
        }
    }
    Ok(())
}

async fn run_index(state: State, opts: IndexOpts) -> anyhow::Result<()> {
    let manager = state.index_manager;

    let init_opts = match opts.recreate {
        true => index::InitOpts::delete_all(),
        false => index::InitOpts::default(),
    };

    manager
        .init(init_opts)
        .await
        .with_context(|| "Failed to initializer Elasticsearch index".to_string())?;
    match opts.post_id {
        Some(post_id) => {
            let post_index = manager.post_index();
            post_index.index_post_by_id(&state.db, &post_id).await?;
        }
        None => {
            manager.index_changes(&state.db, opts.daemon).await?;
        }
    }
    Ok(())
}

async fn run_search(state: State, opts: SearchOpts) -> anyhow::Result<()> {
    let index = state.index_manager.post_index();
    let records = index
        .index()
        .find_records_with_text_query(&opts.query)
        .await?;
    let records: Vec<Record<Post>> = records
        .into_iter()
        .filter_map(|r| r.into_typed_record::<Post>().ok())
        .collect();
    if opts.json {
        println!("{}", serde_json::to_string(&records)?);
    } else {
        debug_print_records(&records[..]);
    }
    Ok(())
}

async fn run_feed(state: State, command: FeedCommand) -> anyhow::Result<()> {
    state.db.init().await?;
    match command {
        FeedCommand::Fetch(opts) => {
            rss::ops::fetch_and_save(&state.db, &opts).await?;
        }
        FeedCommand::Crawl(opts) => {
            rss::ops::crawl_and_save(&state.db, &opts).await?;
        }
        FeedCommand::Watch(_opts) => {
            state.feed_manager.run_watch(state.db).await?;
        }
        FeedCommand::Refetch(opts) => {
            state
                .feed_manager
                .refetch(&state.db, &opts.id_or_url)
                .await?;
        }
    };
    Ok(())
}
