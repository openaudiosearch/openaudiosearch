use clap::Clap;
use elasticsearch::cert::CertificateValidation;
use elasticsearch::{
    auth::Credentials,
    http::transport::{SingleNodeConnectionPool, TransportBuilder},
    indices::{
        IndicesCreateParts, IndicesDeleteParts, IndicesExistsParts, IndicesPutSettingsParts,
    },
    BulkOperation, BulkParts, Elasticsearch, Error, DEFAULT_ADDRESS,
};
use elasticsearch::{GetParts, IndexParts, SearchParts, UpdateByQueryParts};
use http::StatusCode;
use oas_common::types::Post;
use rocket::serde::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::Instant;
use url::Url;

use super::IndexError;
use oas_common::{ElasticMapping, Record, TypedValue, UntypedRecord};

pub const DEFAULT_PREFIX: &str = "oas";

/// ElasticSearch config.
#[derive(Clap, Debug, Clone)]
pub struct Config {
    /// Elasticsearch server URL
    #[clap(long, env = "ELASTICSEARCH_URL")]
    pub url: Option<String>,

    // Elasticsearch index
    // #[clap(long, env = "ELASTICSEARCH_INDEX")]
    // pub index: String,
    /// Elasticsearch index prefix
    #[clap(long, env = "ELASTICSEARCH_PREFIX")]
    pub prefix: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            url: None,
            prefix: None,
        }
    }
}

impl Config {
    /// Creates a new config with server URL and index name.
    pub fn new(url: Option<String>) -> Self {
        Self {
            url,
            // index,
            prefix: None,
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
        let mut url: Url = url.parse()?;
        let first_segment = url
            .path_segments()
            .map(|mut segments| segments.nth(0).map(|s| s.to_string()))
            .flatten();
        let prefix = if let Some(first_segment) = first_segment {
            first_segment.to_string()
        } else {
            DEFAULT_PREFIX.to_string()
        };
        url.set_path("");
        Ok(Self {
            url: Some(url.to_string()),
            prefix: Some(prefix),
        })
    }

    /// Creates config with index name and default values.
    pub fn with_default_url(prefix: String) -> Self {
        Self {
            url: None,
            prefix: Some(prefix),
        }
    }
}

/// ElasticSearch client.
///
/// The client is stateless. It only contains a HTTP client, the index name and the config on how to connect to ElasticSearch.
///
/// We use [elasticsearch-rs](https://github.com/elastic/elasticsearch-rs),
/// you can find the documentation on [docs.rs](https://docs.rs/elasticsearch/7.12.1-alpha.1/elasticsearch/).
#[derive(Debug, Clone)]
pub struct Index {
    client: Arc<Elasticsearch>,
    index: String,
}

impl Index {
    /// Creates a new client with config.
    // pub fn with_config(config: Config) -> Result<Self, Error> {
    //     let client = create_client(config.url)?;
    //     let client = Arc::new(client);
    //     Ok(Self {
    //         client,
    //         index: config.index,
    //     })
    // }

    pub fn with_client_and_name(client: Arc<Elasticsearch>, name: impl ToString) -> Self {
        Self {
            client,
            index: name.to_string(),
        }
    }

    /// Get the index name.
    pub fn index(&self) -> &str {
        &self.index
    }

    /// Get the index name.
    pub fn name(&self) -> &str {
        &self.index
    }

    /// Get the reference to the client.
    pub fn client(&self) -> &Elasticsearch {
        &self.client
    }

    /// Inititialize ElasticSearch.
    ///
    /// This creates the elasticsearch index with the default index mapping if it does not exists. It should be called before calling other
    /// methods on the client.
    pub async fn ensure_index(&self, delete: bool) -> Result<(), IndexError> {
        let mapping = get_default_mapping();
        create_index_if_not_exists(&self.client, &self.index, delete, mapping).await?;
        Ok(())
    }

    pub async fn get_doc<T: DeserializeOwned>(&self, id: &str) -> Result<Option<T>, Error> {
        let res = self
            .client()
            .get(GetParts::IndexId(self.name(), id))
            .send()
            .await?;
        let mut json: serde_json::Value = res.json().await?;
        if !json["found"].as_bool().unwrap() {
            Ok(None)
        } else {
            let doc: T = serde_json::from_value(json["_source"].take())?;
            Ok(Some(doc))
        }
    }

    pub async fn put_doc<T: Serialize>(&self, id: &str, doc: &T) -> Result<(), Error> {
        let _res = self
            .client()
            .index(IndexParts::IndexId(self.name(), id))
            .body(doc)
            .send()
            .await?;
        Ok(())
    }

    /// Put a list of [Record]s to the index.
    ///
    /// Internally the [Record]s are transformed to [UntypedRecord]s, serialized and saved in a
    /// single bulk operation.
    pub async fn put_typed_records<T: TypedValue>(
        &self,
        docs: &[Record<T>],
    ) -> Result<(), IndexError> {
        let docs: Vec<UntypedRecord> = docs
            .iter()
            .filter_map(|r| r.clone().into_untyped_record().ok())
            .collect();
        self.put_untyped_records(&docs).await?;
        Ok(())
    }

    /// Put a list of [UntypedRecord]s to the index
    pub async fn put_untyped_records(&self, docs: &[UntypedRecord]) -> Result<(), IndexError> {
        self.set_refresh_interval(json!("-1")).await?;
        let now = Instant::now();

        index_records(&self.client, &self.index, &docs).await?;

        let duration = now.elapsed();
        let secs = duration.as_secs_f64();

        let _taken = if secs >= 60f64 {
            format!("{}m", secs / 60f64)
        } else {
            format!("{:?}", duration)
        };

        self.set_refresh_interval(json!(null)).await?;
        Ok(())
    }

    /// Update all nested documents on a top-level field with the value from an [UntypedRecord].
    ///
    /// We have a relation from [Post] to [Media](oas_common::types::Media)
    /// Because of this data model we have a nested field on the
    /// elasticsearch level which we have to update as soon as the data arrives.
    /// This function will usually be called with update from a [ChangesStream](crate::couch::changes::ChangesStream).
    ///
    /// See this [Article](https://iridakos.com/programming/2019/05/02/add-update-delete-elasticsearch-nested-objects) for details on working with nested documents in ElasticSearch.
    pub async fn update_nested_record(
        &self,
        field: &str,
        record: &UntypedRecord,
    ) -> Result<(), Error> {
        let script = r#"
def nested_docs = ctx._source[params.field].findAll(nested_doc -> nested_doc['$meta'].guid == params.guid);
for (nested_doc in nested_docs) {
    for (change in params.changes.entrySet()) {
        nested_doc[change.getKey()] = change.getValue() 
    } 
}"#;
        let match_field = format!("{}.$meta.guid", field);
        let match_value = record.guid();
        let body = json!({
          "query": {
            "match": {
              match_field: match_value
            }
          },
          "script": {
              "source": script,
              "params": {
                  "field": field,
                  "guid": record.guid(),
                  "changes": serde_json::to_value(record)?
              }
          }
        });

        let _response = self
            .client
            .update_by_query(UpdateByQueryParts::Index(&[&self.index]))
            .body(body)
            .send()
            .await?;

        Ok(())
    }

    async fn set_refresh_interval(&self, interval: Value) -> Result<(), Error> {
        let response = self
            .client
            .indices()
            .put_settings(IndicesPutSettingsParts::Index(&[&self.index]))
            .body(json!({
                "index" : {
                    "refresh_interval" : interval
                }
            }))
            .send()
            .await?;

        if !response.status_code().is_success() {
            log::error!("Failed to update refresh interval");
        }
        Ok(())
    }

    /// Simple string query on the index.
    pub async fn find_records_with_text_query(
        &self,
        query: &str,
    ) -> Result<Vec<UntypedRecord>, Error> {
        let query = json!({
            "query": { "query_string": { "query": query } }
        });
        let mut response = self
            .client
            .search(SearchParts::Index(&[&self.index]))
            .body(query)
            .pretty(true)
            .send()
            .await?;

        // turn the response into an Error if status code is unsuccessful
        response = response.error_for_status_code()?;

        let json: Value = response.json().await?;
        let records: Vec<UntypedRecord> = json["hits"]["hits"]
            .as_array()
            .unwrap()
            .iter()
            .map(|h| serde_json::from_value(h["_source"].clone()).unwrap())
            .collect();

        Ok(records)
    }
}

/// Check if an Elasticsearch response contains an error, and if so, parse the Elasticsearch
/// exception JSON payload (if possible).
///
/// TODO: Move into an extension trait for Response
async fn check_error(
    response: elasticsearch::http::response::Response,
) -> Result<elasticsearch::http::response::Response, IndexError> {
    let status_code_err = response.error_for_status_code_ref();
    if let Err(status_code_err) = status_code_err {
        let ex = response.exception().await?;
        if let Some(ex) = ex {
            Err(ex.into())
        } else {
            Err(status_code_err.into())
        }
    } else {
        Ok(response)
    }
}

async fn index_records(
    client: &Elasticsearch,
    index_name: &str,
    posts: &[UntypedRecord],
) -> Result<BulkPutResponse, IndexError> {
    if posts.is_empty() {
        return Ok(BulkPutResponse::default());
    }

    let body: Vec<BulkOperation<_>> = posts
        .iter()
        .map(|record| {
            let id = record.id().to_string();
            // let body = serde_json::to_value(record).unwrap();
            BulkOperation::index(record).id(&id).routing(&id).into()
        })
        .collect();

    let response = client
        .bulk(BulkParts::Index(&index_name))
        .body(body)
        .send()
        .await?;

    let response = check_error(response).await?;
    let results: BulkPutResponse = response.json().await?;

    Ok(results)
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct BulkPutResponse {
    took: u32,
    errors: bool,
    items: Vec<BulkPutResponseAction>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BulkPutResponseAction {
    Create(BulkPutResponseItem),
    Delete(BulkPutResponseItem),
    Index(BulkPutResponseItem),
    Update(BulkPutResponseItem),
}

pub type BulkPutResponseItem = oas_common::Object;

async fn create_index_if_not_exists(
    client: &Elasticsearch,
    name: &str,
    delete: bool,
    mapping: serde_json::Value,
) -> Result<(), IndexError> {
    let exists = client
        .indices()
        .exists(IndicesExistsParts::Index(&[&name]))
        .send()
        .await?;

    if exists.status_code().is_success() && delete {
        let delete = client
            .indices()
            .delete(IndicesDeleteParts::Index(&[&name]))
            .send()
            .await?;

        if !delete.status_code().is_success() {
            log::warn!("problem deleting index {}", delete.text().await?);
        } else {
            log::info!("deleted index {}", name);
        }
    }

    if exists.status_code() == StatusCode::NOT_FOUND || delete {
        let response = client
            .indices()
            .create(IndicesCreateParts::Index(&name))
            .body(mapping)
            .send()
            .await?;

        match check_error(response).await {
            Ok(_) => {
                log::info!("created index {}", name);
                Ok(())
            }
            Err(err) => {
                log::error!("error deleting index {}: {}", name, err);
                Err(err)
            }
        }
    } else {
        Ok(())
    }
}

pub fn create_client(addr: Option<String>) -> Result<Elasticsearch, Error> {
    fn default_addr() -> String {
        match std::env::var("ELASTICSEARCH_URL") {
            Ok(server) => server,
            Err(_) => DEFAULT_ADDRESS.into(),
        }
    }

    let url = addr.unwrap_or_else(default_addr);
    let mut url = Url::parse(&url)?;

    let credentials = if url.scheme() == "https" {
        let username = if !url.username().is_empty() {
            let u = url.username().to_string();
            url.set_username("").unwrap();
            u
        } else {
            std::env::var("ES_USERNAME").unwrap_or_else(|_| "elastic".into())
        };

        let password = match url.password() {
            Some(p) => {
                let pass = p.to_string();
                url.set_password(None).unwrap();
                pass
            }
            None => std::env::var("ES_PASSWORD").unwrap_or_else(|_| "changeme".into()),
        };

        Some(Credentials::Basic(username, password))
    } else {
        None
    };

    let conn_pool = SingleNodeConnectionPool::new(url);
    let mut builder = TransportBuilder::new(conn_pool);

    builder = match credentials {
        Some(c) => {
            builder = builder.auth(c);
            builder = builder.cert_validation(CertificateValidation::None);
            builder
        }
        None => builder,
    };

    let transport = builder.build()?;
    Ok(Elasticsearch::new(transport))
}

fn get_default_mapping() -> serde_json::Value {
    let post_mapping = Post::elastic_mapping();
    json!({
        "mappings": {
            "properties": post_mapping
        },
        "settings": {
            "analysis": {
                "analyzer": {
                    "whitespace_plus_delimited": {
                    "tokenizer": "whitespace",
                    "filter": [ "plus_delimited" ]
                    }
                },
                "filter": {
                    "plus_delimited": {
                    "type": "delimited_payload",
                    "delimiter": "|",
                    "encoding": "identity"
                    }
                }
            }
        }
    })
}
