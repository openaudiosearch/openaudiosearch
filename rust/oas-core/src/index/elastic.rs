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
use oas_common::{Record, TypedValue, UntypedRecord};
use rocket::serde::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use url::Url;

use super::IndexError;

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
    mapping: serde_json::Value,
}

impl Index {
    /// Create a new Index client from an Elasticsearch client and index name.
    pub fn new(
        client: Arc<Elasticsearch>,
        name: impl ToString,
        mapping: serde_json::Value,
    ) -> Self {
        Self {
            client,
            index: name.to_string(),
            mapping,
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
        let mapping = wrap_mapping_properties(self.mapping.clone());
        create_index_if_not_exists(&self.client, &self.index, delete, &mapping).await?;
        Ok(())
    }

    pub(super) async fn get_doc<T: DeserializeOwned>(&self, id: &str) -> Result<Option<T>, Error> {
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

    pub(super) async fn put_doc<T: Serialize>(&self, id: &str, doc: &T) -> Result<(), Error> {
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
    ) -> Result<BulkPutResponse, IndexError> {
        let docs: Vec<UntypedRecord> = docs
            .iter()
            .filter_map(|r| r.clone().into_untyped_record().ok())
            .collect();
        self.put_untyped_records(&docs).await
    }

    /// Put a list of [UntypedRecord]s to the index
    pub async fn put_untyped_records(
        &self,
        docs: &[UntypedRecord],
    ) -> Result<BulkPutResponse, IndexError> {
        self.set_refresh_interval(json!("-1")).await?;
        if docs.is_empty() {
            return Ok(BulkPutResponse::default());
        }

        let body: Vec<BulkOperation<_>> = docs
            .iter()
            .map(|record| {
                let id = record.id().to_string();
                // let body = serde_json::to_value(record).unwrap();
                let op = BulkOperation::index(record).id(&id).routing(&id).into();
                op
            })
            .collect();

        let response = self
            .client
            .bulk(BulkParts::Index(&self.index))
            .body(body)
            .send()
            .await?;

        let response = check_error(response).await?;
        let results: BulkPutResponse = response.json().await?;
        log::info!("{}", results.summarize());
        //log::info!("{:#?}", results);

        self.set_refresh_interval(json!(null)).await?;
        Ok(results)
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

    pub async fn query_records<Q: Serialize + std::fmt::Debug>(
        &self,
        query: Q,
    ) -> Result<Vec<UntypedRecord>, Error> {
        // eprintln!("query {:#?}", query);
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
        // eprintln!("res {:#?}", json);
        let records: Vec<UntypedRecord> = json["hits"]["hits"]
            .as_array()
            .unwrap()
            .iter()
            .map(|h| serde_json::from_value(h["_source"].clone()).unwrap())
            .collect();

        Ok(records)
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

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct BulkPutResponse {
    pub took: u32,
    pub errors: bool,
    pub items: Vec<BulkPutResponseAction>,
}

impl BulkPutResponse {
    pub fn stats(&self) -> BulkPutStats {
        let mut stats = BulkPutStats::default();
        for item in self.items.iter() {
            if let Some(error) = &item.inner().error {
                if stats.first_error.is_none() {
                    stats.first_error = Some((item.inner().id.clone(), error.clone()));
                }
                stats.errors += 1;
            } else {
                match item.inner().result {
                    Some(BulkPutResponseResult::Created) => stats.created += 1,
                    Some(BulkPutResponseResult::Updated) => stats.updated += 1,
                    Some(BulkPutResponseResult::Deleted) => stats.deleted += 1,
                    _ => {}
                }
            }
        }
        stats
    }

    // TODO: Remove clones.
    pub fn errors(&self) -> impl Iterator<Item = (String, BulkPutResponseError)> + '_ {
        self.items.iter().filter_map(|ref item| {
            if let Some(ref error) = item.inner().error {
                Some((item.inner().id.to_string(), error.clone()))
            } else {
                None
            }
        })
    }

    fn summarize(&self) -> String {
        let stats = self.stats();
        format!(
            "indexed {} in {}ms: {} errors, {} created, {} updated, {} deleted",
            self.items.len(),
            self.took,
            stats.errors,
            stats.created,
            stats.updated,
            stats.deleted
        )
    }
}

#[derive(Debug, Default)]
pub struct BulkPutStats {
    pub errors: usize,
    pub created: usize,
    pub deleted: usize,
    pub updated: usize,
    pub first_error: Option<(String, BulkPutResponseError)>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BulkPutResponseAction {
    Create(BulkPutResponseItem),
    Delete(BulkPutResponseItem),
    Index(BulkPutResponseItem),
    Update(BulkPutResponseItem),
}

impl BulkPutResponseAction {
    fn inner(&self) -> &BulkPutResponseItem {
        match self {
            Self::Create(item) => &item,
            Self::Delete(item) => &item,
            Self::Index(item) => &item,
            Self::Update(item) => &item,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BulkPutResponseItem {
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(rename = "_index")]
    pub index: String,
    #[serde(rename = "_version")]
    pub version: Option<u64>,
    #[serde(rename = "_seq_no")]
    pub seq_no: Option<u64>,

    #[serde(rename = "_shards")]
    pub shards: Option<serde_json::Value>,

    pub status: u64,
    pub result: Option<BulkPutResponseResult>,
    pub error: Option<BulkPutResponseError>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct BulkPutResponseError {
    pub r#type: String,
    pub reason: String,
    pub index_uuid: Option<String>,
    pub shard: Option<String>,
    pub index: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BulkPutResponseResult {
    Created,
    Deleted,
    Updated,
    NotFound,
}

async fn create_index_if_not_exists(
    client: &Elasticsearch,
    name: &str,
    delete: bool,
    mapping: &serde_json::Value,
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

pub fn wrap_mapping_properties(properties: serde_json::Value) -> serde_json::Value {
    json!({
        "mappings": {
            "properties": properties
        },
        "settings": {
            "analysis": {
                "analyzer": {
                    "payload_delimiter": {
                        "tokenizer": "whitespace",
                        "filter": [ "lowercase", "oas_stemmer", "payload_delimiter_filter" ]
                    }
                },
                "filter": {
                    "oas_stemmer": {
                       "type": "stemmer",
                       "language": "light_german"
                    },
                    "payload_delimiter_filter": {
                        "type": "delimited_payload",
                        "delimiter": "|",
                        "encoding": "identity"
                    }
                }
            }
        }
    })
}
