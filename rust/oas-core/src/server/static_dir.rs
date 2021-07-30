use rocket::http::ContentType;
use rocket::http::{uri::Segments, Method};
use rocket::response::Responder;
use rocket::route::{Handler, Outcome, Route};
use rocket::{Data, Request, Response};
use std::path::PathBuf;

const DEFAULT_RANK: isize = 10;

#[derive(Clone, Debug)]
pub struct IncludedStaticDir {
    rank: isize,
    dir: &'static include_dir::Dir<'static>,
}

impl IncludedStaticDir {
    pub fn new(dir: &'static include_dir::Dir) -> Self {
        Self {
            rank: DEFAULT_RANK,
            dir,
        }
    }
}

impl Into<Vec<Route>> for IncludedStaticDir {
    fn into(self) -> Vec<Route> {
        let mut route = Route::ranked(self.rank, Method::Get, "/<path..>", self);
        route.name = Some(format!("IncludedStaticDir").into());
        vec![route]
    }
}

#[async_trait::async_trait]
impl Handler for IncludedStaticDir {
    async fn handle<'r>(&self, req: &'r rocket::Request<'_>, data: Data<'r>) -> Outcome<'r> {
        use rocket::http::uri::fmt::Path;

        let path = req
            .segments::<Segments<'_, Path>>(0..)
            .ok()
            .and_then(|segments| segments.to_path_buf(true).ok());

        match path {
            Some(mut path) => {
                if path.to_str() == Some("") || path.is_dir() {
                    path.push("index.html");
                }
                serve_from_included_static_dir(req, data, &self.dir, path)
            }
            None => Outcome::forward(data),
        }
    }
}

fn serve_from_included_static_dir<'r>(
    req: &'r rocket::Request<'_>,
    data: Data<'r>,
    dir: &'static include_dir::Dir,
    path: PathBuf,
) -> Outcome<'r> {
    let file = dir.get_file(&path);
    if let Some(file) = file {
        Outcome::from_or_forward(req, data, StaticFile::new(path, file))
    } else {
        Outcome::forward(data)
    }
}

pub struct StaticFile {
    path: PathBuf,
    file: include_dir::File<'static>,
}

impl StaticFile {
    pub fn new(path: PathBuf, file: include_dir::File<'static>) -> Self {
        Self { path, file }
    }
}

impl<'r> Responder<'r, 'static> for StaticFile {
    fn respond_to(self, _req: &'r Request<'_>) -> rocket::response::Result<'static> {
        let body = self.file.contents();
        let len = body.len();
        let body = std::io::Cursor::new(body);
        let mut response = Response::build().sized_body(len, body).finalize();
        if let Some(ext) = self.path.extension() {
            if let Some(ct) = ContentType::from_extension(&ext.to_string_lossy()) {
                response.set_header(ct);
            }
        }

        Ok(response)
    }
}
