use crate::State;
use clap::Parser;
use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::Header;
use rocket::{catchers, Orbit, Request, Rocket};
use rocket_okapi::routes_with_openapi;
use rocket_okapi::swagger_ui::{make_swagger_ui, SwaggerUIConfig};

mod auth;
pub mod error;
mod handlers;
mod proxy;
mod static_dir;

const DEFAULT_PORT: u16 = 8080;
const DEFAULT_HOST: &str = "0.0.0.0";
const FRONTEND_DIR: include_dir::Dir = include_dir::include_dir!("../../frontend/dist");

#[derive(Parser, Default, Clone, Debug)]
pub struct ServerOpts {
    /// Hostname to bind HTTP server to
    #[clap(long, env = "HTTP_HOSTNAME")]
    pub host: Option<String>,
    /// Hostname to bind server to
    #[clap(long, env = "HTTP_PORT")]
    pub port: Option<u16>,
}

pub async fn run_server(mut state: State, opts: ServerOpts) -> anyhow::Result<()> {
    state.init_all().await?;
    let figment = rocket::Config::figment()
        .merge(("port", opts.port.unwrap_or(DEFAULT_PORT)))
        .merge((
            "address",
            opts.host.unwrap_or_else(|| DEFAULT_HOST.to_string()),
        ));

    // TODO: Don't do this default.
    let admin_password = &std::env::var("OAS_ADMIN_PASSWORD").unwrap_or("password".to_string());

    let cors = rocket_cors::CorsOptions::default().to_cors()?;
    let auth = auth::Auth::new();
    auth.ensure_admin_user(admin_password).await;

    let app = rocket::custom(figment)
        .manage(state)
        .manage(auth)
        .attach(cors)
        .attach(OasFairing)
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
                // /post routes
                handlers::post::put_post,
                handlers::post::get_post,
                handlers::post::patch_post,
                handlers::post::post_post,
                // /feed routes
                handlers::feed::put_feed,
                handlers::feed::get_feed,
                handlers::feed::post_feed,
                handlers::feed::get_feeds,
                handlers::feed::delete_feed,
                // /search routes
                handlers::search::search,
                handlers::search::search2,
                // login routes
                auth::post_login,
                auth::get_login,
                auth::logout,
                auth::register,
                auth::private,
                // job routes
                handlers::job::get_all_jobs,
                handlers::job::get_job,
                handlers::job::post_job,
                handlers::job::work_job,
                handlers::job::put_job_completed,
                handlers::job::put_job_failed,
                handlers::job::put_job_progress,
                // changes routes
                handlers::changes::durable_changes,
            ],
        )
        .mount(
            "/swagger-ui/",
            make_swagger_ui(&SwaggerUIConfig {
                url: "../api/v1/openapi.json".to_owned(),
                ..Default::default()
            }),
        )
        .register("/api/v1", catchers![auth::unauthorized]);

    // Mount either a proxy to a frontend dev server,
    // or included static dir.
    let app = match std::env::var("FRONTEND_PROXY") {
        Ok(proxy_addr) => app.mount("/", proxy::ProxyHandler::new(proxy_addr)),
        Err(_) => app.mount("/", static_dir::IncludedStaticDir::new(&FRONTEND_DIR)),
    };

    app.launch().await?;

    Ok(())
}

struct OasFairing;
#[rocket::async_trait]
impl Fairing for OasFairing {
    fn info(&self) -> Info {
        Info {
            name: "OAS logging",
            kind: Kind::Liftoff | Kind::Request,
        }
        /* ... */
    }

    // async fn on_ignite(&self, rocket: Rocket<Build>) -> fairing::Result {
    //     [> ... <]
    // }

    async fn on_liftoff(&self, rocket: &Rocket<Orbit>) {
        let config = rocket.config();
        let proto = config.tls_enabled().then(|| "https").unwrap_or("http");
        let addr = format!("{}://{}:{}", proto, config.address, config.port);
        log::info!("HTTP server listening on {}", addr);
        /* ... */
    }

    async fn on_request(&self, req: &mut Request<'_>, _data: &mut rocket::data::Data<'_>) {
        // Answer API requests with JSON (changes the error handler)
        if req.uri().path().starts_with("/api") {
            req.replace_header(Header::new("accept", "application/json"));
        }
        // req.add_header(Header::new("accept", "application/json"));
        // match req.headers().get_one("accept") {
        //     None | Some(header) if header == "*/*" => {
        //     },
        //     _ => {}
        // }
        // && req.headers().get_one("accept").is_none() {
    }

    // async fn on_response<'r>(&self, req: &'r Request<'_>, res: &mut Response<'r>) {
    //     [> ... <]
    // }
}
