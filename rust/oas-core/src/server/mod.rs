use clap::Clap;
use rocket_okapi::{
    routes_with_openapi,
    swagger_ui::{make_swagger_ui, SwaggerUIConfig},
};

use crate::State;

mod auth;
pub mod error;
mod handlers;
mod proxy;
mod static_dir;

const DEFAULT_PORT: u16 = 8080;
const DEFAULT_HOST: &str = "0.0.0.0";
const FRONTEND_DIR: include_dir::Dir = include_dir::include_dir!("../../frontend/dist");

#[derive(Clap, Default, Clone, Debug)]
pub struct ServerOpts {
    /// Hostname to bind HTTP server to
    #[clap(long, env = "HTTP_HOSTNAME")]
    host: Option<String>,
    /// Hostname to bind server to
    #[clap(long, env = "HTTP_PORT")]
    port: Option<u16>,
}

pub async fn run_server(mut state: State, opts: ServerOpts) -> anyhow::Result<()> {
    state.init_all().await?;
    let figment = rocket::Config::figment()
        .merge(("port", opts.port.unwrap_or(DEFAULT_PORT)))
        .merge((
            "address",
            opts.host.unwrap_or_else(|| DEFAULT_HOST.to_string()),
        ));

    let cors = rocket_cors::CorsOptions::default().to_cors()?;
    let auth = auth::Auth::new();
    let app = rocket::custom(figment)
        .manage(state)
        .manage(auth)
        .attach(cors)
        .mount(
            "/api/v1",
            routes_with_openapi![
                // /record routes
                handlers::record::get_record,
                handlers::record::post_record,
                // /media routes
                handlers::media::put_media,
                handlers::media::get_media,
                handlers::media::patch_media,
                handlers::media::post_media,
                handlers::media::get_media_data,
                // /feed routes
                handlers::feed::put_feed,
                handlers::feed::get_feed,
                handlers::feed::post_feed,
                // /search routes
                handlers::search::search,
                // task routes
                handlers::task::post_transcribe_media,
                // login routes
                auth::login,
                auth::logout,
                auth::private
            ],
        )
        .mount(
            "/swagger-ui/",
            make_swagger_ui(&SwaggerUIConfig {
                url: "../api/v1/openapi.json".to_owned(),
                ..Default::default()
            }),
        );

    // Mount either a proxy to a frontend dev server,
    // or included static dir.
    let app = match std::env::var("FRONTEND_PROXY") {
        Ok(proxy_addr) => app.mount("/", proxy::ProxyHandler::new(proxy_addr)),
        Err(_) => app.mount("/", static_dir::IncludedStaticDir::new(&FRONTEND_DIR)),
    };

    app.launch().await?;

    Ok(())
}
