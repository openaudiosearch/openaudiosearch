use elasticsearch::SearchParts;

use serde_json::json;

use elasticsearch::cert::CertificateValidation;
use elasticsearch::{
    auth::Credentials,
    http::transport::{SingleNodeConnectionPool, TransportBuilder},
    indices::{
        IndicesCreateParts, IndicesDeleteParts, IndicesExistsParts, IndicesPutSettingsParts,
    },
    BulkOperation, BulkParts, Elasticsearch, Error, DEFAULT_ADDRESS,
};
use http::StatusCode;
use serde_json::Value;
use std::time::Instant;
use url::Url;

use oas_common::{Record, TypedValue, UntypedRecord};

pub async fn ensure_index(
    client: &Elasticsearch,
    index_name: &str,
    delete: bool,
) -> Result<(), Error> {
    create_index_if_not_exists(&client, index_name, delete, get_default_mapping()).await?;
    Ok(())
}

pub async fn find_records(
    client: &Elasticsearch,
    index_name: &str,
    query: &str,
) -> Result<Vec<UntypedRecord>, Error> {
    let query = json!({
         "query": { "query_string": { "query": query } }
    });
    let mut response = client
        .search(SearchParts::Index(&[index_name]))
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

pub async fn index_records<T>(
    client: &Elasticsearch,
    index_name: &str,
    docs: &[Record<T>],
) -> Result<(), Error>
where
    T: TypedValue,
{
    set_refresh_interval(&client, &index_name, json!("-1")).await?;
    let now = Instant::now();

    index_posts(&client, index_name, &docs).await?;
    let duration = now.elapsed();
    let secs = duration.as_secs_f64();

    let _taken = if secs >= 60f64 {
        format!("{}m", secs / 60f64)
    } else {
        format!("{:?}", duration)
    };

    set_refresh_interval(&client, &index_name, json!(null)).await?;
    Ok(())
}

async fn set_refresh_interval(
    client: &Elasticsearch,
    index_name: &str,
    interval: Value,
) -> Result<(), Error> {
    let response = client
        .indices()
        .put_settings(IndicesPutSettingsParts::Index(&[index_name]))
        .body(json!({
            "index" : {
                "refresh_interval" : interval
            }
        }))
        .send()
        .await?;

    if !response.status_code().is_success() {
        println!("Failed to update refresh interval");
    }

    Ok(())
}

pub async fn index_posts<T>(
    client: &Elasticsearch,
    index_name: &str,
    posts: &[Record<T>],
) -> Result<(), Error>
where
    T: TypedValue,
{
    let body: Vec<BulkOperation<_>> = posts
        .iter()
        .map(|r| {
            let id = r.id().to_string();
            BulkOperation::index(r).id(&id).routing(&id).into()
        })
        .collect();

    // eprintln!("BODY {:?}", body);
    // eprintln!("JSON: {:?}",

    let response = client
        .bulk(BulkParts::Index(&index_name))
        .body(body)
        .send()
        .await?;

    let json: Value = response.json().await?;
    eprintln!("RES {:?}", json);

    if json["errors"].as_bool().unwrap() {
        let failed: Vec<&Value> = json["items"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|v| !v["error"].is_null())
            .collect();

        // TODO: retry failures
        println!("Errors whilst indexing. Failures: {}", failed.len());
    }

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

pub fn create_client() -> Result<Elasticsearch, Error> {
    fn cluster_addr() -> String {
        match std::env::var("ELASTICSEARCH_URL") {
            Ok(server) => server,
            Err(_) => DEFAULT_ADDRESS.into(),
        }
    }

    let mut url = Url::parse(cluster_addr().as_ref()).unwrap();

    // if the url is https and specifies a username and password, remove from the url and set credentials
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

pub fn get_default_mapping() -> serde_json::Value {
    json!({
        "mappings": {
            "properties": {
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
            }
        }
    })
}
