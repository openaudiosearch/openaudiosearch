use base64::write::EncoderWriter as Base64Encoder;
use clap::Clap;
use oas_common::{Record, TypedValue};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::json;
use serde_json::Value;
use std::collections::HashMap;
use std::io::Write;
use std::sync::Arc;
use std::time;
use surf::http::{headers, mime, Method};
use surf::middleware::{Middleware, Next};
use surf::{Body, Client, Request, RequestBuilder, Response, Url};

// use oas_common::{DecodingError, Record, TypedValue, UntypedRecord};

pub type CouchResult<T> = std::result::Result<T, CouchError>;
// TODO: Remove.
pub type Result<T> = std::result::Result<T, CouchError>;

pub(crate) mod changes;
pub(crate) mod error;
pub(crate) mod types;

pub use changes::ChangesStream;
pub use error::CouchError;
pub use types::*;

/// CouchDB config.
#[derive(Clap, Debug, Clone)]
pub struct Config {
    /// CouchDB hostname
    #[clap(long, env = "COUCHDB_HOST", default_value = "http://localhost:5984")]
    pub host: String,
    /// CouchDB database
    #[clap(long, env = "COUCHDB_DATABASE", default_value = "oas")]
    pub database: String,
    /// CouchDB username
    #[clap(long, env = "COUCHDB_USERNAME")]
    pub user: Option<String>,
    /// CouchDB password
    #[clap(long, env = "COUCHDB_PASSWORD")]
    pub password: Option<String>,
}

/// CouchDB client.
#[derive(Debug, Clone)]
pub struct CouchDB {
    config: Config,
    client: Arc<surf::Client>,
}

impl CouchDB {
    pub fn with_config(config: Config) -> anyhow::Result<Self> {
        // let logger = Logger {};
        let auth = Auth::new(config.clone());
        let base_url = format!("{}/{}/", config.host, config.database);
        let base_url = Url::parse(&base_url)?;
        let mut client = surf::Client::new().with(auth);
        client.set_base_url(base_url);

        Ok(Self {
            config,
            client: Arc::new(client),
        })
    }

    pub async fn init(&self) -> Result<()> {
        let res: Result<Value> = self.send(self.request(Method::Get, "")).await;
        match res {
            Ok(_res) => Ok(()),
            Err(_) => {
                let req = self.request(Method::Put, "").build();
                self.send(req).await
            }
        }
    }

    pub async fn get_all(&self) -> Result<DocList> {
        let mut params = HashMap::new();
        params.insert("include_docs", "true");
        self.get_all_with_params(&params).await
    }

    pub async fn get_all_with_prefix(&self, prefix: &str) -> Result<DocList> {
        let mut params = HashMap::new();
        params.insert("include_docs", "true".to_string());
        if prefix.contains("\"") {
            return Err(CouchError::Other("Prefix may not contain quotes".into()));
        }
        params.insert("startkey", format!("\"{}\"", prefix));
        params.insert("endkey", format!("\"{}{}\"", prefix, "\u{ffff}"));
        self.get_all_with_params(&params).await
    }

    pub async fn get_all_with_params(&self, params: &impl Serialize) -> Result<DocList> {
        // let start = Instant::now();
        let docs: DocList = self
            .send(self.request(Method::Get, "_all_docs").query(params)?)
            .await?;
        // eprintln!("request took: {}", start.elapsed().as_secs_f32());
        // let docs: DocList = serde_json::from_value(docs)?;
        // eprintln!("finalizing took: {}", start.elapsed().as_secs_f32());
        Ok(docs)
    }

    pub async fn get_doc(&self, id: &str) -> Result<Doc> {
        let req = self.request(Method::Get, id).build();
        let doc: Doc = self.send(req).await?;
        Ok(doc)
    }

    pub async fn put_doc(&self, mut doc: Doc) -> Result<PutResponse> {
        let id = doc.id().to_string();
        if doc.rev().is_none() {
            let last_doc = self.get_doc(&id).await;
            if let Ok(last_doc) = last_doc {
                if let Some(rev) = last_doc.rev() {
                    doc.set_rev(Some(rev.to_string()));
                }
            }
        }
        let req = self.request(Method::Put, &id).body(Body::from_json(&doc)?);
        self.send(req).await
    }

    pub async fn put_bulk(&self, docs: Vec<Doc>) -> Result<Vec<PutResult>> {
        let body = serde_json::json!({ "docs": docs });
        let req = self
            .request(Method::Post, "_bulk_docs")
            .body(Body::from_json(&body)?)
            .build();
        let res: Vec<PutResult> = self.send(req).await?;
        let mut errors = 0;
        let mut ok = 0;
        for res in res.iter() {
            match res {
                PutResult::Ok(_) => ok += 1,
                PutResult::Err(_) => errors += 1,
            }
        }
        log::debug!("put {} ({} ok, {} err)", res.len(), ok, errors);
        Ok(res)
    }

    pub async fn put_bulk_update<T>(&self, docs: Vec<T>) -> Result<Vec<PutResult>>
    where
        T: Into<Doc>,
    {
        let mut docs_without_rev = vec![];
        let mut docs: Vec<Doc> = docs.into_iter().map(|doc| doc.into()).collect();
        for (i, doc) in docs.iter().enumerate() {
            if doc.rev().is_none() {
                docs_without_rev.push((doc.id().to_string(), i));
            }
        }
        let req_json: Vec<serde_json::Value> = docs_without_rev
            .iter()
            .map(|(id, _)| json!({ "id": id }))
            .collect();
        let req_json = json!({ "docs": req_json });
        let bulk_get: BulkGetResponse = self
            .send(self.request(Method::Post, "_bulk_get").body(req_json))
            .await?;
        // eprintln!("res: {:#?}", bulk_get);
        for (req_idx, (sent_id, doc_idx)) in docs_without_rev.iter().enumerate() {
            let result = bulk_get.results.get(req_idx);
            let rev = match result {
                Some(BulkGetItem { id, docs }) if id == sent_id && docs.len() == 1 => {
                    let doc = docs.get(0).unwrap();
                    match doc {
                        DocResult::Ok(doc) => doc.rev().map(|s| s.to_string()),
                        DocResult::Err(_err) => None,
                    }
                }
                _ => {
                    return Err(CouchError::Other(
                        "Response does not match request".to_string(),
                    ));
                }
            };
            if let Some(rev) = rev {
                docs.get_mut(*doc_idx).unwrap().set_rev(Some(rev));
            }
        }

        // eprintln!("docs: {:#?}", docs);
        let res = self.put_bulk(docs).await;
        // eprintln!("put res: {:#?}", res);
        res
    }

    pub fn changes(&self, last_seq: Option<String>) -> ChangesStream {
        ChangesStream::new(self.client.clone(), last_seq)
    }

    fn request(&self, method: Method, path: impl AsRef<str>) -> RequestBuilder {
        let url = self.path_to_url(path.as_ref());
        RequestBuilder::new(method, url).content_type(mime::JSON)
    }

    fn path_to_url(&self, path: impl AsRef<str>) -> Url {
        format!(
            "{}/{}/{}",
            self.config.host,
            self.config.database,
            path.as_ref()
        )
        .parse()
        .unwrap()
    }

    async fn send<T>(&self, request: impl Into<Request>) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let mut res = self.client.send(request).await?;
        match res.status().is_success() {
            true => Ok(res.body_json::<T>().await?),
            false => Err(res.body_json::<ErrorDetails>().await?.into()),
        }
    }
}

/// Impls that use Record and TypedValue structs.
impl CouchDB {
    pub async fn get_all_records<T: TypedValue>(&self) -> Result<Vec<Record<T>>> {
        let prefix = format!("{}_", T::NAME);
        let docs = self.get_all_with_prefix(&prefix).await?;
        let records = docs
            .rows
            .into_iter()
            .filter_map(|doc| doc.doc.into_typed_record::<T>().ok())
            .collect();
        Ok(records)
    }

    pub async fn put_record<T: TypedValue>(&self, record: Record<T>) -> Result<PutResponse> {
        let doc = Doc::from_typed_record(record);
        self.put_doc(doc).await
    }
}

#[derive(Debug)]
pub struct Auth {
    config: Config,
}
impl Auth {
    pub fn new(config: Config) -> Self {
        Self { config }
    }
}

#[surf::utils::async_trait]
impl Middleware for Auth {
    async fn handle(
        &self,
        mut req: Request,
        client: Client,
        next: Next<'_>,
    ) -> surf::Result<Response> {
        if let Some(username) = &self.config.user {
            let mut header_value = b"Basic ".to_vec();
            {
                let mut encoder = Base64Encoder::new(&mut header_value, base64::STANDARD);
                // The unwraps here are fine because Vec::write* is infallible.
                write!(encoder, "{}:", username).unwrap();
                if let Some(password) = &self.config.password {
                    write!(encoder, "{}", password).unwrap();
                }
                let header_value = encoder.finish().unwrap().to_vec();
                let header_value = String::from_utf8(header_value).unwrap();
                req.set_header(headers::AUTHORIZATION, header_value);
            }
        }
        let res = next.run(req, client).await?;
        Ok(res)
    }
}

#[derive(Debug)]
pub struct Logger;

#[surf::utils::async_trait]
impl Middleware for Logger {
    async fn handle(&self, req: Request, client: Client, next: Next<'_>) -> surf::Result<Response> {
        log::debug!("[req] {} {}", req.method(), req.url().path());
        // let body = req.take_body();
        // let string = body.into_string().await?;
        // if string.len() > 0 {
        //     eprintln!("body: {}", string);
        //     req.body_string(string);
        // }

        let now = time::Instant::now();
        let res = next.run(req, client).await?;
        log::debug!("[res] {} ({:?})", res.status(), now.elapsed());
        Ok(res)
    }
}
