use clap::Clap;
use rocket::{get, routes};

use crate::State;

const DEFAULT_PORT: u16 = 8080;
const DEFAULT_HOST: &str = "0.0.0.0";

pub mod error;
mod handlers;

#[derive(Clap, Default, Clone, Debug)]
pub struct ServerOpts {
    /// Hostname to bind HTTP server to
    #[clap(long, env = "HTTP_HOSTNAME")]
    host: Option<String>,
    /// Hostname to bind server to
    #[clap(long, env = "HTTP_PORT")]
    port: Option<u16>,
}

pub async fn run_server(state: State, opts: ServerOpts) -> anyhow::Result<()> {
    let figment = rocket::Config::figment()
        .merge(("port", opts.port.unwrap_or(DEFAULT_PORT)))
        .merge(("address", opts.host.unwrap_or(DEFAULT_HOST.to_string())));

    let cors = rocket_cors::CorsOptions::default().to_cors()?;
    let app = rocket::custom(figment)
        .manage(state)
        //.attach(cors::Cors)
        .attach(cors)
        // debug routes
        .mount("/hello", routes![world])
        // api routes
        .mount("/api/v1/record", handlers::record::routes())
        .mount("/api/v1/media", handlers::media::routes())
        .mount("/api/v1/feed", handlers::feed::routes())
        .mount("/api/v1/search", handlers::search::routes())
        // legacy routes
        .mount("/oas/v1/search", handlers::search::routes())
        .mount("/oas/v1/feed", handlers::feed::routes())
        .mount("/oas/v1", handlers::legacy::routes());

    app.launch().await?;

    Ok(())
}

#[get("/world")]
fn world() -> &'static str {
    "Hello, world!"
}
