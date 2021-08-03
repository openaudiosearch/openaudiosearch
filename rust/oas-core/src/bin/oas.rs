use clap::Clap;
use futures::stream::StreamExt;
use oas_common::types::Media;
use oas_common::Record;
use oas_core::rss::manager::run_manager;
use oas_core::server::{run_server, ServerOpts};
use oas_core::types::Post;
use oas_core::util::debug_print_records;
use oas_core::State;
use oas_core::{couch, index, rss, tasks};
use tokio::task;
use url::Url;

#[derive(Clap)]
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
    #[clap(long, env = "REDIS_URL")]
    pub redis_url: Option<String>,

    /// Bind HTTP server to host
    #[clap(long, env = "HTTP_HOST")]
    pub http_host: Option<String>,

    /// Bind HTTP server to port
    #[clap(long, env = "HTTP_PORT")]
    pub http_port: Option<u16>,
}

#[derive(Clap)]
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
    /// Create a test task
    Task(tasks::TaskOpts),
    /// Run the HTTP API server
    Server(ServerOpts),
    /// Run all services
    Run(ServerOpts),
}

#[derive(Clap)]
struct FeedCommands {
    /// Subcommand
    #[clap(subcommand)]
    command: FeedCommand,
}

#[derive(Clap)]
enum FeedCommand {
    /// Fetch a feed by URL.
    Fetch(FeedFetchOpts),
    /// Fetch and crawl a feed by URL (increasing offset param).
    Crawl(rss::ops::CrawlOpts),
    /// Watch on CouchDB changes stream for new feeds
    Watch,
}

#[derive(Clap)]
struct FeedFetchOpts {
    /// Feed URL
    url: Url,
}

#[derive(Clap)]
struct IndexOpts {
    /// Run forever in daemon mode
    #[clap(short, long)]
    daemon: bool,

    /// Delete and recreate the index
    #[clap(long)]
    recreate: bool,
}

impl IndexOpts {
    pub fn run_forever() -> Self {
        Self {
            daemon: true,
            recreate: false,
        }
    }
}

#[derive(Clap)]
struct WatchOpts {
    /// Rev to start the watch stream at.
    since: Option<String>,
}

#[derive(Clap)]
struct SearchOpts {
    /// Search query
    query: String,
    /// Print results as JSON
    #[clap(short, long)]
    json: bool,
}

#[derive(Clap)]
struct ListOpts {
    /// ID prefix (type)
    prefix: Option<String>,
    // Output as JSON
    // #[clap(long)]
    // json: bool,
}

fn setup() -> anyhow::Result<()> {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "oas=info")
    }
    env_logger::init();
    Ok(())
}

#[async_std::main]
async fn main() -> anyhow::Result<()> {
    setup()?;
    let args = Args::parse();

    let db = couch::CouchDB::with_url(args.couchdb_url.as_deref())?;
    let index_manager = index::IndexManager::with_url(args.elasticsearch_url.as_deref())?;
    let tasks = tasks::CeleryManager::with_url(args.redis_url.as_deref())?;

    let state = State {
        db,
        index_manager,
        tasks,
    };

    let result = match args.command {
        Command::Watch(opts) => run_watch(state, opts).await,
        Command::List(opts) => run_list(state, opts).await,
        Command::Debug => run_debug(state).await,
        Command::Index(opts) => run_index(state, opts).await,
        Command::Search(opts) => run_search(state, opts).await,
        Command::Feed(opts) => run_feed(state, opts.command).await,
        Command::Task(opts) => run_task(state, opts).await,
        Command::Server(opts) => run_server(state, opts).await,
        Command::Run(server_opts) => run_all(state, server_opts).await,
    };
    result
}

async fn run_all(mut state: State, server_opts: ServerOpts) -> anyhow::Result<()> {
    state.init_all().await?;
    let server_task = task::spawn(run_server(state.clone(), server_opts));
    let index_task = task::spawn(run_index(state.clone(), IndexOpts::run_forever()));
    let feed_task = task::spawn(run_feed(state, FeedCommand::Watch));

    // This calls std::process::exit() on ctrl_c signal.
    // TODO: We might need cancel signals into the tasks for some tasks.
    let exit_task = task::spawn(run_exit());

    server_task.await??;
    index_task.await??;
    feed_task.await??;
    exit_task.await?;
    Ok(())
}

async fn run_exit() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to listen for cancel event");
    std::process::exit(0)
}

async fn run_task(state: State, opts: tasks::TaskOpts) -> anyhow::Result<()> {
    tasks::run_celery(state, opts).await?;
    Ok(())
}

async fn run_list(state: State, opts: ListOpts) -> anyhow::Result<()> {
    let db = state.db;
    let docs = match opts.prefix {
        Some(prefix) => db.get_all_with_prefix(&prefix).await?,
        None => db.get_all().await?,
    };
    let len = docs.rows.len();
    for doc in docs.rows() {
        println!("{}", serde_json::to_string(&doc).unwrap());
    }
    log::info!("total {}", len);
    Ok(())
}

async fn run_debug(_state: State) -> anyhow::Result<()> {
    eprintln!("OAS debug -- nothing here");
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

    manager.init(init_opts).await?;
    manager.index_changes(&state.db, opts.daemon).await?;
    Ok(())
}

async fn run_search(state: State, opts: SearchOpts) -> anyhow::Result<()> {
    let index = state.index_manager.data_index();
    let records = index.find_records_with_text_query(&opts.query).await?;
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
            rss::ops::fetch_and_save(&state.db, &opts.url).await?;
        }
        FeedCommand::Crawl(opts) => {
            rss::ops::crawl_and_save(&state.db, &opts).await?;
        }
        FeedCommand::Watch => {
            run_manager(&state.db).await?;
        }
    };
    Ok(())
}
