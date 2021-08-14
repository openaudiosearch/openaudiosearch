use anyhow::Context;
use clap::Clap;
use oas_common::UntypedRecord;
use oas_common::{Record, TypedValue};
use reqwest::{Method, RequestBuilder};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::json;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use url::Url;

pub type CouchResult<T> = std::result::Result<T, CouchError>;
// TODO: Remove.
pub type Result<T> = std::result::Result<T, CouchError>;

pub(crate) mod changes;
pub(crate) mod error;
mod manager;
pub mod resolver;
mod table;
pub(crate) mod types;

pub use changes::ChangesStream;
pub use error::CouchError;
pub use manager::*;
pub use types::*;

use self::table::Table;

pub const DEFAULT_DATABASE: &str = "oas";
pub const DEFAULT_HOST: &str = "http://localhost:5984";
// pub const DEFAULT_PORT: u16 = 5984;

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

impl Default for Config {
    fn default() -> Self {
        Self {
            host: DEFAULT_HOST.to_string(),
            database: DEFAULT_DATABASE.to_string(),
            user: Some("admin".to_string()),
            password: Some("password".to_string()),
        }
    }
}

impl Config {
    pub fn with_defaults(database: String) -> Self {
        Self {
            host: DEFAULT_HOST.into(),
            database,
            ..Default::default()
        }
    }

    pub fn from_url_or_default(url: Option<&str>) -> anyhow::Result<Self> {
        if let Some(url) = url {
            Self::from_url(&url)
        } else {
            Ok(Self::default())
        }
    }

    pub fn from_url(url: &str) -> anyhow::Result<Self> {
        let url: Url = url
            .parse()
            .with_context(|| format!("Failed to parse CouchDB URL"))?;
        let host = if let Some(host) = url.host() {
            if let Some(port) = url.port() {
                format!("{}://{}:{}", url.scheme(), host, port)
            } else {
                format!("{}://{}", url.scheme(), host)
            }
        } else {
            DEFAULT_HOST.to_string()
        };
        let user = if !url.username().is_empty() {
            Some(url.username().to_string())
        } else {
            None
        };
        let password = url.password().map(|p| p.to_string());
        let first_segment = url
            .path_segments()
            .map(|mut segments| segments.nth(0).map(|s| s.to_string()))
            .flatten();
        let database = if let Some(first_segment) = first_segment {
            first_segment.to_string()
        } else {
            DEFAULT_DATABASE.to_string()
        };
        let config = Config {
            host,
            database,
            user,
            password,
        };
        Ok(config)
    }
}

/// CouchDB client.
///
/// The client is stateless. It only contains a HTTP client and the config on how to connect to a
/// database.
#[derive(Debug, Clone)]
pub struct CouchDB {
    config: Arc<Config>,
    client: Arc<reqwest::Client>,
}

impl CouchDB {
    /// Create a new client with config.
    /// TODO: Remove Ok-wrapping
    pub fn with_config(config: Config) -> anyhow::Result<Self> {
        let client = reqwest::Client::new();

        Ok(Self {
            config: Arc::new(config),
            client: Arc::new(client),
        })
    }

    pub fn with_config_and_client(config: Config, client: reqwest::Client) -> Self {
        Self {
            config: Arc::new(config),
            client: Arc::new(client),
        }
    }

    /// Create a new client with a CouchDB URL.
    ///
    /// The URL should have the following format:
    /// http://username:password@hostname:5984/dbname
    /// If passing None for url a client will be created with the default address
    /// http://localhost:5984/oas
    pub fn with_url<S>(url: Option<S>) -> anyhow::Result<Self>
    where
        S: AsRef<str>,
    {
        let url = url.map(|s| s.as_ref().to_string());
        let config = Config::from_url_or_default(url.as_deref())?;
        let db = Self::with_config(config)?;
        Ok(db)
    }

    /// Init the database.
    ///
    /// This creates the database if it does not exists. It should be called before calling other
    /// methods on the client.
    pub async fn init(&self) -> anyhow::Result<()> {
        let res: Result<Value> = self.send(self.request(Method::GET, "")).await;
        match res {
            Ok(_res) => {
                log::trace!("check database {}: ok", self.config.database);
                Ok(())
            }
            Err(_) => {
                let req = self.request(Method::PUT, "");
                let _res: serde_json::Value = self.send(req).await.with_context(|| {
                    format!("Failed to create database {}", self.config.database)
                })?;
                Ok(())
            }
        }
    }

    pub async fn destroy_and_init(&self) -> Result<()> {
        let req = self.request(Method::DELETE, "");
        let _: Result<()> = self.send(req).await;
        let req = self.request(Method::PUT, "");
        self.send(req).await
    }

    /// Get all docs from the database.
    pub async fn get_all(&self) -> Result<DocList> {
        let mut params = HashMap::new();
        params.insert("include_docs", "true");
        self.get_all_with_params(&params).await
    }

    /// Get many docs by their ID from the database.
    pub async fn get_many(&self, ids: &[&str]) -> Result<DocList> {
        if ids.is_empty() {
            return Ok(DocList::default());
        }
        let mut params = HashMap::new();
        params.insert("include_docs", serde_json::to_string(&true).unwrap());
        // let keys: String = ids.join(",");
        // params.insert("keys", serde_json::to_value(keys).unwrap());
        params.insert("keys", serde_json::to_string(ids).unwrap());
        self.get_all_with_params(&params).await
    }

    /// Get all docs where the couch id starts with a prefix.
    ///
    /// When the ids contain a type prefix (e.g. "oas.Media_someidstring", then
    /// this method can be used to get all docs with a type.
    pub async fn get_all_with_prefix(&self, prefix: &str) -> Result<DocList> {
        let mut params = HashMap::new();
        params.insert("include_docs", "true".to_string());
        if prefix.contains('\"') {
            return Err(CouchError::Other("Prefix may not contain quotes".into()));
        }
        params.insert("startkey", format!("\"{}\"", prefix));
        params.insert("endkey", format!("\"{}{}\"", prefix, "\u{ffff}"));
        self.get_all_with_params(&params).await
    }

    /// Get all docs while passing a map of params.
    pub async fn get_all_with_params(&self, params: &impl Serialize) -> Result<DocList> {
        let req = self.request(Method::GET, "_all_docs").query(params);
        let docs: Value = self.send(req).await?;
        let docs: DocList = serde_json::from_value(docs)?;
        Ok(docs)
    }

    /// Get a doc from the id by its id.
    pub async fn get_doc(&self, id: &str) -> Result<Doc> {
        let req = self.request(Method::GET, id);
        let doc: Doc = self.send(req).await?;
        Ok(doc)
    }

    /// Put a doc into the database.
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
        let req = self.request(Method::PUT, &id).json(&doc);
        self.send(req).await
    }

    /// Put a list of docs into the database in a single bulk operation.
    pub async fn put_bulk(&self, docs: Vec<Doc>) -> Result<Vec<PutResult>> {
        let body = serde_json::json!({ "docs": docs });
        let req = self.request(Method::POST, "_bulk_docs").json(&body);
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

    /// Put a list of docs into the database in a single bulk operation, while first fetching the
    /// latest rev for each doc.
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
        let req = self.request(Method::POST, "_bulk_get").json(&req_json);
        let bulk_get: BulkGetResponse = self.send(req).await?;
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

        let res = self.put_bulk(docs).await;
        res
    }

    /// Get a stream of changes from the database.
    ///
    /// Some options can be set on the ChangesStream, see [ChangesStream].
    ///
    /// Example:
    /// ```no_run
    /// # use oas_core::couch::{Config,CouchDB};
    /// # use futures::stream::StreamExt;
    /// # async fn run() -> anyhow::Result<()> {
    /// let config = Config::with_defaults("some_db".into());
    /// let db = CouchDB::with_config(config)?;
    /// let mut stream = db.changes(None);
    /// while let Some(event) = stream.next().await {
    ///     let event = event.unwrap();
    ///     if let Some(doc) = event.doc {
    ///         eprintln!("new doc or rev: {:?}", doc);
    ///     }
    /// }
    /// # Ok(())
    /// # };
    /// ```
    pub fn changes(&self, last_seq: Option<String>) -> ChangesStream {
        ChangesStream::new(self.clone(), last_seq)
    }

    pub async fn get_last_seq(&self) -> anyhow::Result<String> {
        let mut params = HashMap::new();
        params.insert("descending", "true".to_string());
        params.insert("limit", "1".to_string());
        let path = "_changes";
        let req = self.request(Method::GET, path).query(&params);
        let res: GetChangesResult = self.send(req).await?;
        Ok(res.last_seq)
    }

    fn request(&self, method: Method, path: impl AsRef<str>) -> RequestBuilder {
        let url = self.path_to_url(path.as_ref());
        let builder = self.client.request(method, url);
        let builder = if let Some(username) = &self.config.user {
            builder.basic_auth(username, self.config.password.as_ref())
        } else {
            builder
        };
        builder
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

    async fn send<T>(&self, request: RequestBuilder) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let res = request.send().await?;
        // let mut res = self.client.execute(request).await?;
        match res.status().is_success() {
            true => Ok(res.json::<T>().await?),
            false => Err(CouchError::Couch(
                res.status(),
                res.json::<ErrorDetails>().await?,
            )),
        }
    }
}

impl CouchDB {
    pub fn table<T: TypedValue>(&self) -> Table<T> {
        Table::new(self.clone())
    }
}

/// Methods on the CouchDB client that directly take or return [Record]s.
impl CouchDB {
    /// Get all records with the type from the database.
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

    /// Get a single record by its type and id.
    ///
    /// ```ignore
    /// let record = db.get_record::<Media>("someidstring").await?;
    /// ```
    pub async fn get_record<T: TypedValue>(&self, id: &str) -> Result<Record<T>> {
        // let id = T::guid(id);
        let doc = self.get_doc(&id).await?;
        let record = doc.into_typed_record::<T>()?;
        Ok(record)
    }

    pub async fn get_many_records<T: TypedValue>(&self, ids: &[&str]) -> Result<Vec<Record<T>>> {
        // let ids: Vec<String> = ids.iter().map(|id| T::guid(id)).collect();
        // let ids: Vec<&str> = ids.iter().map(|id| id.as_str()).collect();
        let rows = self
            .get_many(&ids[..])
            .await?
            .rows
            .into_iter()
            .filter_map(|doc| doc.doc.into_typed_record::<T>().ok())
            .collect();
        Ok(rows)
    }

    pub async fn get_many_records_untyped(&self, ids: &[&str]) -> Result<Vec<UntypedRecord>> {
        let rows = self
            .get_many(ids)
            .await?
            .rows
            .into_iter()
            .filter_map(|doc| doc.doc.into_untyped_record().ok())
            .collect();
        Ok(rows)
    }

    /// Put a single record into the database.
    pub async fn put_record<T: TypedValue>(&self, record: Record<T>) -> Result<PutResponse> {
        let doc = Doc::from_typed_record(record);
        self.put_doc(doc).await
    }

    /// Put a vector of records into the database in a single operation.
    pub async fn put_record_bulk<T: TypedValue>(
        &self,
        records: Vec<Record<T>>,
    ) -> Result<Vec<PutResult>> {
        let docs = records.into_iter().map(Doc::from_typed_record).collect();
        self.put_bulk(docs).await
    }

    /// Put a vector of untyped records into the database in a single operation.
    pub async fn put_untyped_record_bulk(
        &self,
        records: Vec<UntypedRecord>,
    ) -> Result<Vec<PutResult>> {
        let docs = records.into_iter().map(Doc::from_untyped_record).collect();
        self.put_bulk(docs).await
    }

    /// Put a vector of records into the database in a single operation,
    /// while first fetching the lastest rev for records that do not have a rev set.
    pub async fn put_record_bulk_update<T: TypedValue>(
        &self,
        records: Vec<Record<T>>,
    ) -> Result<Vec<PutResult>> {
        let docs = records.into_iter().map(Doc::from_typed_record).collect();
        self.put_bulk_update(docs).await
    }

    /// Put a vector of untyped records into the database in a single operation.
    pub async fn put_untyped_record_bulk_update(
        &self,
        records: Vec<UntypedRecord>,
    ) -> Result<Vec<PutResult>> {
        let docs = records.into_iter().map(Doc::from_untyped_record).collect();
        self.put_bulk_update(docs).await
    }
}
