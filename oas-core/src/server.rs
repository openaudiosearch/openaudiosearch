use clap::Clap;
use std::net::{SocketAddr};



use actix_web::middleware::Logger;
use actix_web::{get, web, App, HttpRequest, HttpServer};

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
    tokio::task::spawn_blocking(move || run_server_actix(opts)).await??;
    Ok(())
}

#[actix_web::main]
pub async fn run_server_actix(opts: ServerOpts) -> anyhow::Result<()> {
    run_server_inner(opts).await?;
    Ok(())
}
pub async fn run_server_inner(opts: ServerOpts) -> anyhow::Result<()> {
    let host = opts.host.unwrap_or(DEFAULT_HOST.to_string());
    let port = opts.port.unwrap_or(DEFAULT_PORT);
    let addr: SocketAddr = format!("{}:{}", host, port).parse()?;

    let _server = HttpServer::new(move || {
        let app = App::new().wrap(Logger::default()).service(index);
        app
    })
    .bind(addr)?
    .run()
    .await?;
    Ok(())
}

#[get("/resource1/{name}/index.html")]
async fn index(req: HttpRequest, name: web::Path<String>) -> String {
    println!("REQ: {:?}", req);
    format!("Hello: {}!\r\n", name)
}
