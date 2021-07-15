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
use elasticsearch::{SearchParts, UpdateByQueryParts};
use http::StatusCode;
use oas_common::types::Post;
use serde_json::{json, Value};
use std::time::Instant;
use url::Url;

use oas_common::{ElasticMapping, Record, TypedValue, UntypedRecord};

// ElasticSearch config.
#[derive(Clap, Debug, Clone)]
pub struct Config {
    /// Elasticsearch server URL
    #[clap(long, env = "ELASTICSEARCH_URL")]
    pub url: Option<String>,

    /// Elasticsearch index
    #[clap(long, env = "ELASTICSEARCH_INDEX")]
    pub index: String,
}

impl Config {
    // creates a new config with <url> and <index>
    pub fn new(url: Option<String>, index: String) -> Self {
        Self { url, index }
    }
    // creates default config
    pub fn with_default_url(index: String) -> Self {
        Self { url: None, index }
    }
}

/// ElasticSearch client.
///
/// The client is stateless. It only contains a HTTP client and the config on how to connect to a
/// ElasticSearch. We use [elasticsearch-rs](https://github.com/elastic/elasticsearch-rs), 
/// you can find the documentation on [docs.rs](https://docs.rs/elasticsearch/7.12.1-alpha.1/elasticsearch/)
#[derive(Debug, Clone)]
pub struct Index {
    client: Elasticsearch,
    index: String,
}

impl Index {
    /// Create a new client with config.
    pub fn with_config(config: Config) -> Result<Self, Error> {
        let client = create_client(config.url)?;
        Ok(Self {
            client,
            index: config.index,
        })
    }
    /// Get the index name
    pub fn index(&self) -> &str {
        &self.index
    }
    /// Get the reference to the client
    pub fn client(&self) -> &Elasticsearch {
        &self.client
    }

    /// Inititialize ElasticSearch.
    ///
    /// This creates the elasticsearch index with the default index mapping if it does not exists. It should be called before calling other
    /// methods on the client.
    pub async fn ensure_index(&self, delete: bool) -> Result<(), Error> {
        let mapping = get_default_mapping();
        create_index_if_not_exists(&self.client, &self.index, delete, mapping).await?;
        Ok(())
    }

    /// Put a list of [Record]s to the ElasticSearch index.
    /// Internally the [Record]s a transformed to [UntypedRecord]s.
    pub async fn put_typed_records<T: TypedValue>(&self, docs: &[Record<T>]) -> Result<(), Error> {
        let docs: Vec<UntypedRecord> = docs
            .iter()
            .filter_map(|r| r.clone().into_untyped_record().ok())
            .collect();
        self.put_untyped_records(&docs).await?;
        Ok(())
    }
    /// Put a list of [UntypedRecord]s to the elasticsearch index
    pub async fn put_untyped_records(&self, docs: &[UntypedRecord]) -> Result<(), Error> {
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

    /// We have a relation from [Post] to [oas_common::types::Media]
    /// Because of this data model we have a nested field on the 
    /// elasticsearch level which we have to update as soon as the data arrives.
    /// This function will called from the [crate::couch::changes::ChangesStream]. 
    /// We adapt this mostly from the example in this [Article](https://iridakos.com/programming/2019/05/02/add-update-delete-elasticsearch-nested-objects) from [iridakos](https://iridakos.com/) 
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

        let response = self
            .client
            .update_by_query(UpdateByQueryParts::Index(&[&self.index]))
            .body(body)
            .send()
            .await?;

        eprintln!("response {:?}", response);

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

    /// Simple string query on the ElasticSearch index 
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

async fn index_records(
    client: &Elasticsearch,
    index_name: &str,
    posts: &[UntypedRecord],
) -> Result<(), Error> {
    let body: Vec<BulkOperation<_>> = posts
        .iter()
        .map(|r| {
            let id = r.id().to_string();
            BulkOperation::index(r).id(&id).routing(&id).into()
        })
        .collect();

    let response = client
        .bulk(BulkParts::Index(&index_name))
        .body(body)
        .send()
        .await?;

    let json: Value = response.json().await?;

    let len = json["items"].as_array().unwrap().len();

    if json["errors"].as_bool().unwrap() {
        let failed: Vec<&Value> = json["items"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|v| !v["error"].is_null())
            .collect();

        // TODO: retry failures
        log::error!("Errors whilst indexing. Failures: {}", failed.len());
    }
    log::info!("Indexed {} records", len);

    Ok(())
}

async fn create_index_if_not_exists(
    client: &Elasticsearch,
    name: &str,
    delete: bool,
    mapping: serde_json::Value,
) -> Result<(), Error> {
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
            println!("Problem deleting index: {}", delete.text().await?);
        } else {
            println!("Deleted index: {}", delete.text().await?);
        }
    }

    if exists.status_code() == StatusCode::NOT_FOUND || delete {
        let response = client
            .indices()
            .create(IndicesCreateParts::Index(&name))
            .body(mapping)
            .send()
            .await?;

        if !response.status_code().is_success() {
            println!("Error while creating index");
        } else {
            println!("Created index: {}", name);
        }
    }

    Ok(())
}

fn create_client(addr: Option<String>) -> Result<Elasticsearch, Error> {
    fn default_addr() -> String {
        match std::env::var("ELASTICSEARCH_URL") {
            Ok(server) => server,
            Err(_) => DEFAULT_ADDRESS.into(),
        }
    }

    let url = addr.unwrap_or(default_addr());
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
            // {
                // "type": {
                //     "type": "keyword"
                // },
                // "id": {
                //     "type": "integer"
                // },
        //         "parent_id": {
        //             "relations": {
        //                 "question": "answer"
        //             },
        //             "type": "join"
        //         },
        //         "creation_date": {
        //             "type": "date"
        //         },
        //         "score": {
        //             "type": "integer"
        //         },
        //         "body": {
        //             "analyzer": "html",
        //             "search_analyzer": "expand",
        //             "type": "text"
        //         },
        //         "owner_user_id": {
        //             "type": "integer"
        //         },
        //         "owner_display_name": {
        //             "type": "keyword"
        //         },
        //         "last_editor_user_id": {
        //             "type": "integer"
        //         },
        //         "last_edit_date": {
        //             "type": "date"
        //         },
        //         "last_activity_date": {
        //             "type": "date"
        //         },
        //         "comment_count": {
        //             "type": "integer"
        //         },
        //         "title": {
        //             "analyzer": "expand",
        //             "norms": false,
        //             "fields": {
        //                 "raw": {
        //                     "type": "keyword"
        //                 }
        //             },
        //             "type": "text"
        //         },
        //         "title_suggest": {
        //             "type": "completion"
        //         },
        //         "accepted_answer_id": {
        //             "type": "integer"
        //         },
        //         "view_count": {
        //             "type": "integer"
        //         },
        //         "last_editor_display_name": {
        //             "type": "keyword"
        //         },
        //         "tags": {
        //             "type": "keyword"
        //         },
        //         "answer_count": {
        //             "type": "integer"
        //         },
        //         "favorite_count": {
        //             "type": "integer"
        //         },
        //         "community_owned_date": {
        //             "type": "date"
        //         }
        //     },
        //     "_routing": {
        //         "required": true
        //     },
        //     "_source": {
        //         "excludes": ["title_suggest"]
        //     }
        // },
        // "settings": {
        //     "index.number_of_shards": 3,
        //     "index.number_of_replicas": 0,
        //     "analysis": {
        //         "analyzer": {
        //             "html": {
        //                 "char_filter": ["html_strip", "programming_language"],
        //                 "filter": ["lowercase", "stop"],
        //                 "tokenizer": "standard",
        //                 "type": "custom"
        //             },
        //             "expand": {
        //                 "char_filter": ["programming_language"],
        //                 "filter": ["lowercase", "stop"],
        //                 "tokenizer": "standard",
        //                 "type": "custom"
        //             }
        //         },
        //         "char_filter": {
        //             "programming_language": {
        //                 "mappings": [
        //                     "c# => csharp", "C# => csharp",
        //                     "f# => fsharp", "F# => fsharp",
        //                 ],
        //                 "type": "mapping"
        //             }
        //         }
            // }
        }
    })
}
