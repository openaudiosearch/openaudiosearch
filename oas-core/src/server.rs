use clap::Clap;

use rocket::{get, routes};

const DEFAULT_PORT: u16 = 8080;
const DEFAULT_HOST: &str = "127.0.0.1";

#[derive(Clap, Default, Clone, Debug)]
pub struct ServerOpts {
    /// Hostname to bind HTTP server to
    #[clap(long)]
    host: Option<String>,
    /// Hostname to bind server to
    #[clap(long)]
    port: Option<u16>,
}

pub async fn run_server(opts: ServerOpts) -> anyhow::Result<()> {
    let figment = rocket::Config::figment()
        .merge(("port", opts.port.unwrap_or(DEFAULT_PORT)))
        .merge(("address", opts.host.unwrap_or(DEFAULT_HOST.to_string())));

    let app = rocket::custom(figment).mount("/hello", routes![world]);

    app.launch().await?;

    Ok(())
}

#[get("/world")]
fn world() -> &'static str {
    "Hello, world!"
}
