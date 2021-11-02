use futures::{ready, FutureExt, StreamExt, TryStreamExt};
use futures::{Future, Stream};
use futures_batch::ChunksTimeoutStreamExt;
use oas_common::UntypedRecord;
use reqwest::{Method, Response};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::{self, Duration};
use tokio::io::AsyncBufReadExt;
use tokio::time::{sleep, Sleep};
use tokio_stream::wrappers::LinesStream;
use tokio_util::io::StreamReader;
use tracing::{error, warn};
use tracing_attributes::instrument;

use crate::couch::ErrorDetails;

use super::types::{ChangeEvent, Event};
use super::{CouchDB, CouchError, CouchResult};

pub const BATCH_TIMEOUT: time::Duration = time::Duration::from_millis(200);
pub const BATCH_MAX_LEN: usize = 1000;

/// The max timeout value for longpoll/continous HTTP requests
/// that CouchDB supports (see [1]).
///
/// [1]: https://docs.couchdb.org/en/stable/api/database/changes.html
const COUCH_MAX_TIMEOUT: usize = 60000;

/// The stream for the `_changes` endpoint.
///
/// This is returned from [Database::changes].
#[derive(Debug)]
pub struct ChangesStream {
    last_seq: Option<String>,
    db: CouchDB,
    state: ChangesStreamState,
    params: HashMap<String, String>,
    body: Option<serde_json::Value>,
    infinite: bool,
    retries: usize,
    max_retries: usize,
    retry_timeout: Duration,
}

enum ChangesStreamState {
    Retrying(Pin<Box<Sleep>>),
    Idle,
    Requesting(Pin<Box<dyn Future<Output = CouchResult<Response>> + Send + 'static>>),
    Reading(Pin<Box<dyn Stream<Item = io::Result<String>> + Send + 'static>>),
}

impl fmt::Debug for ChangesStreamState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Retrying(_) => write!(f, "Retrying"),
            Self::Idle => write!(f, "Idle"),
            Self::Requesting(_) => write!(f, "Requesting"),
            Self::Reading(_) => write!(f, "Reading"),
        }
    }
}

impl ChangesStream {
    /// Create a new changes stream.
    pub fn new(db: CouchDB, last_seq: Option<String>) -> Self {
        let mut params = HashMap::new();
        params.insert("feed".to_string(), "continuous".to_string());
        params.insert("timeout".to_string(), "0".to_string());
        params.insert("include_docs".to_string(), "true".to_string());
        Self::with_params(db, last_seq, params)
    }

    /// Create a new changes stream with params.
    pub fn with_params(
        db: CouchDB,
        last_seq: Option<String>,
        params: HashMap<String, String>,
    ) -> Self {
        Self {
            db,
            params,
            state: ChangesStreamState::Idle,
            infinite: false,
            last_seq,
            retries: 0,
            max_retries: 10,
            retry_timeout: Duration::from_millis(100),
            body: None,
        }
    }

    /// Set the starting seq.
    pub fn set_last_seq(&mut self, last_seq: Option<String>) {
        self.last_seq = last_seq;
    }

    /// Set infinite mode.
    ///
    /// If set to true, the changes stream will wait and poll for changes. Otherwise,
    /// the stream will return all changes until now and then close.
    pub fn set_infinite(&mut self, infinite: bool) {
        self.infinite = infinite;
        let timeout = match infinite {
            true => COUCH_MAX_TIMEOUT.to_string(),
            false => 0.to_string(),
        };
        self.params.insert("timeout".to_string(), timeout);
    }

    /// Set a selector to filter the changes stream.
    ///
    /// See https://docs.couchdb.org/en/stable/api/database/changes.html#selector for details.
    pub fn set_selector(&mut self, selector: serde_json::Value) {
        self.set_param("filter", "_selector");
        self.body = Some(selector);
    }

    pub fn set_param(&mut self, key: impl ToString, value: impl ToString) {
        self.params.insert(key.to_string(), value.to_string());
    }

    /// Get the last retrieved seq.
    pub fn last_seq(&self) -> &Option<String> {
        &self.last_seq
    }

    /// Whether this stream is running in infinite mode.
    pub fn infinite(&self) -> bool {
        self.infinite
    }

    // pub fn batched_records<T: TypedValue>(self) -> RecordChangesStream<T> {
    //     RecordChangesStream::new(self)
    // }

    pub fn batched_untyped_records(
        self,
        opts: BatchOpts,
    ) -> impl Stream<Item = UntypedRecordBatch> {
        // let batch_timeout = BATCH_TIMEOUT;
        // let batch_max_len = BATCH_MAX_LEN;
        let changes = self.chunks_timeout(opts.max_len, opts.timeout);
        let changes = changes.map(|batch| UntypedRecordBatch {
            last_seq: get_last_seq(&batch[..]),
            records: changes_into_untyped_records(batch),
        });
        changes
    }
}

pub struct BatchOpts {
    pub timeout: time::Duration,
    pub max_len: usize,
}
impl Default for BatchOpts {
    fn default() -> Self {
        Self {
            timeout: BATCH_TIMEOUT,
            max_len: BATCH_MAX_LEN,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, JsonSchema, Clone)]
pub struct UntypedRecordBatch {
    pub records: Vec<UntypedRecord>,
    pub last_seq: Option<String>,
}

impl UntypedRecordBatch {
    pub fn len(&self) -> usize {
        self.records.len()
    }
    pub fn last_seq(&self) -> Option<&str> {
        self.last_seq.as_deref()
    }

    pub fn records(&self) -> &[UntypedRecord] {
        &self.records[..]
    }

    pub fn into_inner(self) -> Vec<UntypedRecord> {
        self.records
    }
}

async fn get_changes(
    db: CouchDB,
    params: HashMap<String, String>,
    _body: Option<serde_json::Value>,
) -> CouchResult<Response> {
    // let req = db.request(Method::POST, "_changes").query(&params);
    let req = db.request(Method::GET, "_changes").query(&params);
    // let req = if let Some(body) = body {
    //     req.json(&body)
    // } else {
    //     req
    // };
    let res = req.send().await?;
    Ok(res)
}

impl Stream for ChangesStream {
    type Item = CouchResult<ChangeEvent>;
    #[instrument(skip(self, cx))]
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            let retries_exceeded = self.retries >= self.max_retries;
            match self.state {
                ChangesStreamState::Retrying(ref mut sleep) => {
                    if retries_exceeded {
                        return Poll::Ready(None);
                    }
                    ready!(sleep.as_mut().poll(cx));
                    self.retries += 1;
                    self.state = ChangesStreamState::Idle;
                }
                ChangesStreamState::Idle => {
                    let mut params = self.params.clone();
                    if let Some(seq) = &self.last_seq {
                        params.insert("since".to_string(), seq.clone());
                    }
                    let body = self.body.clone();
                    let fut = get_changes(self.db.clone(), params, body);
                    self.state = ChangesStreamState::Requesting(Box::pin(fut));
                }
                ChangesStreamState::Requesting(ref mut fut) => match ready!(fut.poll_unpin(cx)) {
                    Err(err) => {
                        error!(err = %err, "Request failed: {}", err);
                        self.state =
                            ChangesStreamState::Retrying(Box::pin(sleep(self.retry_timeout)));
                        return Poll::Ready(Some(Err(err)));
                    }
                    Ok(res) => match res.status().is_success() {
                        true => {
                            let stream = res
                                .bytes_stream()
                                .map_err(|err| io::Error::new(io::ErrorKind::Other, err));
                            let reader = StreamReader::new(stream);
                            let lines = Box::pin(LinesStream::new(reader.lines()));
                            self.state = ChangesStreamState::Reading(lines)
                        }
                        false => {
                            self.state =
                                ChangesStreamState::Retrying(Box::pin(sleep(self.retry_timeout)));
                            return Poll::Ready(Some(Err(CouchError::Couch(
                                res.status(),
                                ErrorDetails::new(
                                    res.status().canonical_reason().unwrap(),
                                    "",
                                    None,
                                ),
                            ))));
                        }
                    },
                },
                ChangesStreamState::Reading(ref mut lines) => {
                    let line = ready!(lines.poll_next_unpin(cx));
                    match line {
                        None => {
                            self.state = ChangesStreamState::Idle;
                        }
                        Some(Err(err)) => {
                            let message = format!("{}", err);
                            let inner = err
                                .into_inner()
                                .and_then(|err| err.downcast::<reqwest::Error>().ok());
                            match inner {
                                Some(reqwest_err) if reqwest_err.is_timeout() && self.infinite => {
                                    self.state = ChangesStreamState::Idle;
                                }
                                Some(reqwest_err) => {
                                    self.state = ChangesStreamState::Retrying(Box::pin(sleep(
                                        self.retry_timeout,
                                    )));
                                    return Poll::Ready(Some(Err(CouchError::Http(*reqwest_err))));
                                }
                                _ => {
                                    self.state = ChangesStreamState::Retrying(Box::pin(sleep(
                                        self.retry_timeout,
                                    )));
                                    return Poll::Ready(Some(Err(CouchError::Other(format!(
                                        "{}",
                                        message
                                    )))));
                                }
                            }
                        }
                        Some(Ok(line)) if line.is_empty() => continue,
                        Some(Ok(line)) => match serde_json::from_str::<Event>(&line) {
                            Ok(Event::Change(event)) => {
                                self.last_seq = Some(event.seq.clone());
                                self.retries = 0;
                                return Poll::Ready(Some(Ok(event)));
                            }
                            Ok(Event::Finished(event)) => {
                                self.last_seq = Some(event.last_seq.clone());
                                self.state = ChangesStreamState::Idle;
                                if !self.infinite {
                                    return Poll::Ready(None);
                                }
                            }
                            Err(e) => {
                                self.state = ChangesStreamState::Retrying(Box::pin(sleep(
                                    self.retry_timeout,
                                )));
                                return Poll::Ready(Some(Err(e.into())));
                            }
                        },
                    }
                }
            }
        }
    }
}

pub fn changes_into_untyped_records(batch: Vec<CouchResult<ChangeEvent>>) -> Vec<UntypedRecord> {
    let records: Vec<_> = batch
        .into_iter()
        .filter_map(|ev| {
            ev.ok()
                .and_then(|ev| ev.doc)
                .and_then(|doc| doc.into_untyped_record().ok())
        })
        .collect();
    records
}

fn get_last_seq(batch: &[CouchResult<ChangeEvent>]) -> Option<String> {
    batch.last().and_then(|v| match v {
        Ok(v) => Some(v.seq.to_string()),
        _ => None,
    })
}

// #[cfg(test)]
// mod tests {
//     use crate::couch::CouchDB;
//     use futures::StreamExt;
//     use serde_json::{json, Value};
//     #[tokio::test]
//     async fn should_get_changes() {
//         let client = Client::new_local_test().unwrap();
//         let db = client.db("should_get_changes").await.unwrap();
//         let mut changes = db.changes(None);
//         changes.set_infinite(true);
//         let t = tokio::spawn({
//             let db = db.clone();
//             async move {
//                 let mut docs: Vec<Value> = (0..10)
//                     .map(|idx| {
//                         json!({
//                             "_id": format!("test_{}", idx),
//                             "count": idx,
//                         })
//                     })
//                     .collect();

//                 db.bulk_docs(&mut docs)
//                     .await
//                     .expect("should insert 10 documents");
//             }
//         });

//         let mut collected_changes = vec![];
//         while let Some(change) = changes.next().await {
//             collected_changes.push(change);
//             if collected_changes.len() == 10 {
//                 break;
//             }
//         }
//         assert!(collected_changes.len() == 10);
//         t.await.unwrap();
//     }
// }
