use base64::write::EncoderWriter as Base64Encoder;
use futures::io::AsyncBufReadExt;
use futures::io::Lines;
use futures::ready;
use futures::Future;
use futures::FutureExt;
use futures::Stream;
use futures::StreamExt;
use log::*;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;
use std::io::Write;
use std::pin::Pin;
use std::str::FromStr;
use std::sync::Arc;
use std::task::Context;
use std::task::Poll;
use std::time;
use surf::http::Mime;
use surf::http::{headers, mime, Method};
use surf::middleware::{Middleware, Next};
use surf::{Body, Client, Request, RequestBuilder, Response, Url};
use thiserror::Error;

use oas_common::{DecodingError, Record, TypedValue, UntypedRecord};

pub type Result<T> = std::result::Result<T, CouchError>;

/// CouchDB config.
#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub database: String,
    pub user: Option<String>,
    pub password: Option<String>,
}

/// CouchDB client.
#[derive(Debug, Clone)]
pub struct CouchDB {
    config: Config,
    client: Arc<surf::Client>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocMeta {
    #[serde(rename = "_id")]
    id: String,
    #[serde(skip_serializing_if = "is_null")]
    #[serde(rename = "_rev")]
    rev: Option<String>,
    // #[serde(skip_serializing_if = "is_null")]
    // #[serde(rename = "_revs_info")]
    // revs_info: Option<Vec<RevInfo>>,
    // #[serde(skip_serializing_if = "is_null")]
    // #[serde(rename = "_local_seq")]
    // local_seq: Option<u32>,
}

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct RevInfo {
//     rev: String,
//     status: String,
// }

impl DocMeta {
    pub fn new(id: String, rev: Option<String>) -> Self {
        Self { id, rev }
    }
    pub fn with_id(id: String) -> Self {
        Self { id, rev: None }
    }
    pub fn with_id_and_rev(id: String, rev: String) -> Self {
        Self { id, rev: Some(rev) }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocList {
    pub total_rows: u32,
    pub offset: u32,
    pub rows: Vec<DocListEntry>,
}

impl DocList {
    pub fn into_typed_docs<T: DeserializeOwned>(self) -> serde_json::Result<Vec<T>> {
        let results: serde_json::Result<Vec<T>> = self
            .rows
            .into_iter()
            .map(|d| d.doc.into_typed::<T>())
            .collect();
        results
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocListEntry {
    pub id: String,
    pub key: String,
    pub doc: Doc,
}

pub type Object = serde_json::Map<String, serde_json::Value>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Doc {
    #[serde(flatten)]
    pub meta: DocMeta,
    #[serde(flatten)]
    pub doc: Object,
}

impl Doc {
    pub fn into_typed<T: DeserializeOwned>(self) -> serde_json::Result<T> {
        let doc: T = serde_json::from_value(serde_json::Value::Object(self.doc))?;
        Ok(doc)
    }

    pub fn from_typed<T>(meta: DocMeta, doc: T) -> anyhow::Result<Self>
    where
        T: Serialize,
    {
        let doc = serde_json::to_value(doc)?;
        if let serde_json::Value::Object(doc) = doc {
            Ok(Self::new(meta, doc))
        } else {
            Err(anyhow::anyhow!("Not an object"))
        }
    }

    pub fn from_typed_record<T>(record: Record<T>) -> Self
    where
        T: TypedValue,
    {
        let meta = DocMeta::with_id(record.guid().to_string());
        let doc = record.into_json_object().unwrap();
        Self { meta, doc }
    }

    pub fn into_untyped_record<T>(self) -> serde_json::Result<UntypedRecord> {
        let record: UntypedRecord = serde_json::from_value(serde_json::Value::Object(self.doc))?;
        Ok(record)
    }

    pub fn into_typed_record<T>(self) -> std::result::Result<Record<T>, DecodingError>
    where
        T: TypedValue,
    {
        let record: UntypedRecord = serde_json::from_value(serde_json::Value::Object(self.doc))?;
        let record: Record<T> = record.into_typed_record()?;
        Ok(record)
    }

    pub fn new(meta: DocMeta, doc: Object) -> Self {
        Self { meta, doc }
    }

    pub fn id(&self) -> &str {
        &self.meta.id
    }
    pub fn rev(&self) -> Option<&str> {
        self.meta.rev.as_deref()
    }
    pub fn set_rev(&mut self, rev: Option<String>) {
        self.meta.rev = rev;
    }
}

impl<T> From<Record<T>> for Doc
where
    T: TypedValue,
{
    fn from(record: Record<T>) -> Self {
        Doc::from_typed_record(record)
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Event {
    Change(ChangeEvent),
    Finished(FinishedEvent),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChangeEvent {
    pub seq: String,
    pub id: String,
    pub changes: Vec<Change>,

    #[serde(default)]
    pub deleted: bool,

    #[serde(default)]
    pub doc: Option<Doc>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Change {
    pub rev: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FinishedEvent {
    pub last_seq: String,
    pub pending: Option<u64>, // not available on CouchDB 1.0
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PutResponse {
    pub id: String,
    pub ok: bool,
    pub rev: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ErrorDetails {
    error: String,
    id: Option<String>,
    reason: String,
}

#[derive(Error, Debug)]
pub enum CouchError {
    #[error("HTTP error: {0}")]
    Http(surf::Error),
    #[error("CouchDB error")]
    Couch(#[from] ErrorDetails),
    #[error("Serialization error")]
    Json(#[from] serde_json::Error),
    #[error("IO error")]
    IO(#[from] std::io::Error),
    #[error("Error: {0}")]
    Other(String),
}

impl From<surf::Error> for CouchError {
    fn from(err: surf::Error) -> Self {
        Self::Http(err)
    }
}

impl std::error::Error for ErrorDetails {}

impl fmt::Display for ErrorDetails {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(id) = &self.id {
            write!(
                f,
                "CouchDB error for id {}: {} (reason: {})",
                id, self.error, self.reason
            )
        } else {
            write!(f, "CouchDB error: {} (reason: {})", self.error, self.reason)
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum PutResult {
    Ok(PutResponse),
    Err(ErrorDetails),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BulkGetResponse {
    results: Vec<BulkGetItem>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BulkGetItem {
    id: String,
    docs: Vec<DocResult>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum DocResult {
    #[serde(rename = "ok")]
    Ok(Doc),
    #[serde(rename = "error")]
    Err(ErrorDetails),
}

impl CouchDB {
    pub fn with_config(config: Config) -> anyhow::Result<Self> {
        let logger = Logger {};
        let auth = Auth::new(config.clone());
        let base_url = format!("{}/{}/", config.host, config.database);
        let base_url = Url::parse(&base_url)?;
        let mut client = surf::Client::new().with(logger).with(auth);
        client.set_base_url(base_url);

        Ok(Self {
            config,
            client: Arc::new(client),
        })
    }

    pub async fn init(&self) -> Result<()> {
        let req = self.request(Method::Get, "").build();
        let res: Result<Value> = self.send(req).await;
        match res {
            Ok(res) => {
                eprintln!("res: {}", res);
            }
            Err(_) => {
                let req = self.request(Method::Put, "").build();
                self.send(req).await?;
            }
        }
        Ok(())
    }

    pub async fn get_doc(&self, id: &str) -> Result<Doc> {
        let req = self.request(Method::Get, id).build();
        let doc: Doc = self.send(req).await?;
        // let params = HashMap::new();
        // let doc: Doc = self.get(format!("{}", id), None).await?;
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
        self.send(req).await
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
                    return Err(
                        CouchError::Other("Response does not match request".to_string()).into(),
                    );
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

    pub fn changes_stream(&self, last_seq: Option<String>) -> ChangesStream {
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

    pub async fn post<T: DeserializeOwned>(
        &self,
        url: impl AsRef<str>,
        body: impl Serialize,
    ) -> surf::Result<T> {
        let mut req = self.client.post(url).build();
        req.body_json(&body)?;
        req.set_content_type(Mime::from_str("application/json")?);
        let res = self.client.recv_string(req).await?;
        // eprintln!("res: {}", res);
        let res = serde_json::from_str(&res)?;
        Ok(res)
    }
    pub async fn get<T: DeserializeOwned>(
        &self,
        url: impl AsRef<str>,
        params: Option<HashMap<String, String>>,
    ) -> surf::Result<T> {
        let mut req = self.client.get(url).build();
        if let Some(params) = params {
            req.set_query(&params)?;
        }
        let res = self.client.recv_string(req).await?;
        eprintln!("res: {}", res);
        let res = serde_json::from_str(&res)?;
        Ok(res)
    }
}

pub struct ChangesStream {
    last_seq: Option<String>,
    client: Arc<Client>,
    state: ChangesStreamState,
    params: HashMap<String, String>,
    infinite: bool,
}

enum ChangesStreamState {
    Idle,
    Requesting(Pin<Box<dyn Future<Output = surf::Result<Response>>>>),
    Reading(Lines<Response>),
}

impl ChangesStream {
    pub fn new(client: Arc<Client>, last_seq: Option<String>) -> Self {
        let mut params = HashMap::new();
        params.insert("feed".to_string(), "continuous".to_string());
        params.insert("timeout".to_string(), "0".to_string());
        params.insert("include_docs".to_string(), "true".to_string());
        Self::with_params(client, last_seq, params)
    }

    pub fn with_params(
        client: Arc<Client>,
        last_seq: Option<String>,
        params: HashMap<String, String>,
    ) -> Self {
        Self {
            client,
            params,
            state: ChangesStreamState::Idle,
            infinite: false,
            last_seq,
        }
    }

    pub fn set_last_seq(&mut self, last_seq: Option<String>) {
        self.last_seq = last_seq;
    }

    pub fn set_infinite(&mut self, infinite: bool) {
        self.infinite = infinite;
        let timeout = match infinite {
            true => "60000".to_string(),
            false => "0".to_string(),
        };
        self.params.insert("timeout".to_string(), timeout);
    }

    pub fn last_seq(&self) -> &Option<String> {
        &self.last_seq
    }

    pub fn infinite(&self) -> bool {
        self.infinite
    }
}

async fn get_changes(
    client: Arc<Client>,
    params: HashMap<String, String>,
) -> surf::Result<Response> {
    let req = client.get("_changes").query(&params).unwrap().build();
    let res = client.send(req).await;
    res
}

impl Stream for ChangesStream {
    type Item = Result<ChangeEvent>;
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            self.state = match self.state {
                ChangesStreamState::Idle => {
                    let mut params = self.params.clone();
                    if let Some(seq) = &self.last_seq {
                        params.insert("since".to_string(), seq.clone());
                    }
                    let fut = get_changes(self.client.clone(), params);
                    ChangesStreamState::Requesting(Box::pin(fut))
                }
                ChangesStreamState::Requesting(ref mut fut) => match ready!(fut.poll_unpin(cx)) {
                    Err(e) => return Poll::Ready(Some(Err(e.into()))),
                    Ok(res) => match res.status().is_success() {
                        true => ChangesStreamState::Reading(res.lines()),
                        false => {
                            return Poll::Ready(Some(Err(surf::Error::new(
                                res.status(),
                                anyhow::anyhow!(res.status().canonical_reason()),
                            )
                            .into())));
                        }
                    },
                },
                ChangesStreamState::Reading(ref mut lines) => {
                    let line = ready!(lines.poll_next_unpin(cx));
                    match line {
                        None => ChangesStreamState::Idle,
                        Some(Err(e)) => return Poll::Ready(Some(Err(e.into()))),
                        Some(Ok(line)) if line.len() == 0 => continue,
                        Some(Ok(line)) => match serde_json::from_str::<Event>(&line) {
                            Ok(Event::Change(event)) => {
                                self.last_seq = Some(event.seq.clone());
                                // eprintln!("event {:?}", event);
                                return Poll::Ready(Some(Ok(event)));
                            }
                            Ok(Event::Finished(event)) => {
                                self.last_seq = Some(event.last_seq.clone());
                                if !self.infinite {
                                    return Poll::Ready(None);
                                }
                                // eprintln!("event {:?}", event);
                                ChangesStreamState::Idle
                            }
                            Err(e) => {
                                // eprintln!("Decoding error {} on line {}", e, line);
                                return Poll::Ready(Some(Err(e.into())));
                            }
                        },
                    }
                }
            }
        }
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
    async fn handle(
        &self,
        mut req: Request,
        client: Client,
        next: Next<'_>,
    ) -> surf::Result<Response> {
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

fn is_null<T: Serialize>(t: &T) -> bool {
    serde_json::to_value(t)
        .unwrap_or(serde_json::Value::Null)
        .is_null()
}
