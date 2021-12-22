use futures::stream::StreamExt;
use reqwest::header;
use rocket::http::Status;
use rocket::http::{HeaderMap, Method};
use rocket::request::FromRequest;
use rocket::response::stream::{ByteStream, ReaderStream};
use rocket::route::{Handler, Outcome, Route};
use rocket::Data;
use std::ops::Deref;

/// List of headers to forward on proxy requests.
pub static HEADERS_RESPONSE: [reqwest::header::HeaderName; 10] = [
    header::CONTENT_TYPE,
    header::CONTENT_LENGTH,
    header::CONTENT_RANGE,
    header::ACCEPT_RANGES,
    header::PRAGMA,
    header::EXPIRES,
    header::DATE,
    header::CACHE_CONTROL,
    header::ETAG,
    header::LAST_MODIFIED,
];

/// List of headers to forward on proxy requests.
pub static HEADERS_REQUEST: [reqwest::header::HeaderName; 9] = [
    reqwest::header::RANGE,
    reqwest::header::CONTENT_TYPE,
    reqwest::header::IF_MATCH,
    reqwest::header::IF_RANGE,
    reqwest::header::IF_MODIFIED_SINCE,
    reqwest::header::IF_UNMODIFIED_SINCE,
    reqwest::header::ETAG,
    reqwest::header::ACCEPT,
    reqwest::header::ACCEPT_ENCODING,
];

/// Responder struct to forward a reqwest response as a stream
/// while copying the status code and some headers.
pub struct ReqwestResponse {
    res: reqwest::Response,
}

impl ReqwestResponse {
    pub fn new(res: reqwest::Response) -> Self {
        Self { res }
    }
}

impl From<reqwest::Response> for ReqwestResponse {
    fn from(res: reqwest::Response) -> Self {
        Self::new(res)
    }
}

impl<'r> rocket::response::Responder<'r, 'r> for ReqwestResponse {
    fn respond_to(self, _: &'r rocket::Request<'_>) -> rocket::response::Result<'r> {
        let mut res = rocket::response::Response::build().finalize();
        // Copy over header values.
        copy_response_headers(&self.res, &mut res, &HEADERS_RESPONSE);
        // Copy status code.
        res.set_status(rocket::http::Status::new(self.res.status().as_u16()));

        // Pass through the body stream, with some required type juggling.
        // TODO: The reqwest body stream can error at any type. Returning None in the map will
        // likely work, but rocket's byte stream also has a shutdown mechanism that might be more
        // appropriate.
        let stream = self.res.bytes_stream();
        let stream = stream.filter_map(|item| async move { item.ok() });
        let stream = ByteStream::from(stream);
        let stream = stream.0.map(std::io::Cursor::new);
        res.set_streamed_body(ReaderStream::from(stream));
        Ok(res)
    }
}

/// Copy headers from a rocket request into a reqwest request.
pub fn copy_request_headers(
    in_headers: &rocket::http::HeaderMap,
    out_req: &mut reqwest::Request,
    headers: &[reqwest::header::HeaderName],
) {
    for header_name in headers {
        for header_value in in_headers.get(header_name.as_str()) {
            let value = header_value.parse::<reqwest::header::HeaderValue>();
            if let Ok(value) = value {
                out_req.headers_mut().append(header_name, value);
            }
        }
    }
}

/// Copy headers from a reqwest response into a rocket response.
pub fn copy_response_headers(
    in_res: &reqwest::Response,
    out_res: &mut rocket::response::Response,
    headers: &[reqwest::header::HeaderName],
) {
    for header_name in headers {
        if let Some(header_value) = in_res.headers().get(header_name) {
            if let Ok(header_value) = String::from_utf8(header_value.as_bytes().to_vec()) {
                out_res.set_header(rocket::http::Header::new(
                    header_name.to_string(),
                    header_value,
                ));
            }
        }
    }
}

const DEFAULT_RANK: isize = 5;

/// A route handler that proxies all requests to a target http or https server.
#[derive(Clone)]
pub struct ProxyHandler {
    client: reqwest::Client,
    target: String,
    rank: isize,
}

impl From<ProxyHandler> for Vec<Route> {
    fn from(proxy: ProxyHandler) -> Self {
        let mut routes = vec![
            Route::ranked(proxy.rank, Method::Get, "/<path..>", proxy.clone()),
            Route::ranked(proxy.rank, Method::Put, "/<path..>", proxy.clone()),
            Route::ranked(proxy.rank, Method::Post, "/<path..>", proxy.clone()),
            Route::ranked(proxy.rank, Method::Patch, "/<path..>", proxy.clone()),
            Route::ranked(proxy.rank, Method::Head, "/<path..>", proxy.clone()),
            Route::ranked(proxy.rank, Method::Delete, "/<path..>", proxy.clone()),
        ];
        for route in routes.iter_mut() {
            route.name = Some(format!("Proxy({})", proxy.target).into());
        }
        routes
    }
}

impl ProxyHandler {
    /// Create a proxy handler to the specified target URL.
    /// The incoming request's path and querystring will be appended to this URL.
    /// It should not contain a trailing slash.
    pub fn new(target: String) -> Self {
        Self::with_rank(target, DEFAULT_RANK)
    }

    pub fn with_rank(target: String, rank: isize) -> Self {
        Self {
            client: reqwest::Client::new(),
            target,
            rank,
        }
    }
}

fn convert_method_rocket_reqwest(method: rocket::http::Method) -> reqwest::Method {
    reqwest::Method::from_bytes(method.as_str().as_bytes()).unwrap()
}

#[async_trait::async_trait]
impl Handler for ProxyHandler {
    async fn handle<'r>(&self, req: &'r rocket::Request<'_>, data: Data<'r>) -> Outcome<'r> {
        let url = format!("{}{}", self.target, req.uri());
        let method = convert_method_rocket_reqwest(req.method());
        let out_req = self.client.request(method, url).build();
        let mut out_req = match out_req {
            Ok(req) => req,
            Err(_err) => return Outcome::failure(Status::BadGateway),
        };
        copy_request_headers(req.headers(), &mut out_req, &HEADERS_REQUEST);
        let res = self.client.execute(out_req).await;
        match res {
            Ok(res) => Outcome::from_or_forward(req, data, ReqwestResponse::new(res)),
            Err(_err) => Outcome::failure(Status::BadGateway),
        }
    }
}

/// Request guard to get a request's headers.
pub struct Headers<'r>(pub &'r HeaderMap<'r>);

impl<'r> Deref for Headers<'r> {
    type Target = HeaderMap<'r>;
    fn deref(&self) -> &Self::Target {
        self.0
    }
}

#[async_trait::async_trait]
impl<'r> FromRequest<'r> for Headers<'r> {
    type Error = ();
    async fn from_request(
        request: &'r rocket::Request<'_>,
    ) -> rocket::request::Outcome<Self, Self::Error> {
        let headers = request.headers();
        rocket::request::Outcome::Success(Self(headers))
    }
}
