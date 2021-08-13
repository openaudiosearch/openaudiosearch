use futures::{ready, FutureExt, StreamExt, TryStreamExt};
use futures::{Future, Stream};
use futures_batch::{ChunksTimeout, ChunksTimeoutStreamExt};
use oas_common::{Record, TypedValue, UntypedRecord};
use reqwest::{Method, Response};
use std::collections::HashMap;
use std::io;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time;
use tokio::io::AsyncBufReadExt;
use tokio_stream::wrappers::LinesStream;
use tokio_util::io::StreamReader;

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
pub struct ChangesStream {
    last_seq: Option<String>,
    db: CouchDB,
    state: ChangesStreamState,
    params: HashMap<String, String>,
    infinite: bool,
}

enum ChangesStreamState {
    Idle,
    Requesting(Pin<Box<dyn Future<Output = CouchResult<Response>> + Send + 'static>>),
    Reading(Pin<Box<dyn Stream<Item = io::Result<String>> + Send + 'static>>),
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

    /// Get the last retrieved seq.
    pub fn last_seq(&self) -> &Option<String> {
        &self.last_seq
    }

    /// Whether this stream is running in infinite mode.
    pub fn infinite(&self) -> bool {
        self.infinite
    }

    pub fn batched_records<T: TypedValue>(self) -> RecordChangesStream<T> {
        RecordChangesStream::new(self)
    }

    pub fn batched_untyped_records(self) -> UntypedRecordChangesStream {
        UntypedRecordChangesStream::new(self)
    }
}

pub struct UntypedRecordChangesStream {
    changes: ChunksTimeout<ChangesStream>,
    last_seq: Option<String>,
}

pub struct RecordChangesStream<T> {
    changes: ChunksTimeout<ChangesStream>,
    last_seq: Option<String>,
    typ: PhantomData<T>,
}

impl UntypedRecordChangesStream {
    pub fn new(changes: ChangesStream) -> Self {
        let timeout = BATCH_TIMEOUT;
        let max_len = BATCH_MAX_LEN;
        let changes = changes.chunks_timeout(max_len, timeout);
        Self {
            changes,
            last_seq: None,
        }
    }

    pub fn last_seq(&self) -> Option<&str> {
        self.last_seq.as_deref()
    }

    fn poll_next(&mut self, cx: &mut Context<'_>) -> Poll<Option<Vec<UntypedRecord>>> {
        let changes = ready!(self.changes.poll_next_unpin(cx));
        match changes {
            None => Poll::Ready(None),
            Some(changes) => {
                let last_seq = get_last_seq(&changes[..]);
                let records: Vec<_> = changes
                    .into_iter()
                    .filter_map(|ev| {
                        ev.ok()
                            .and_then(|ev| ev.doc)
                            .and_then(|doc| doc.into_untyped_record().ok())
                    })
                    .collect();
                self.last_seq = last_seq;
                Poll::Ready(Some(records))
            }
        }
    }
}

impl Stream for UntypedRecordChangesStream {
    type Item = Vec<UntypedRecord>;
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.get_mut().poll_next(cx)
    }
}

impl<T> Stream for RecordChangesStream<T>
where
    T: TypedValue + Unpin,
{
    type Item = Vec<Record<T>>;
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.get_mut().poll_next(cx)
    }
}

impl<T> RecordChangesStream<T>
where
    T: TypedValue,
{
    pub fn new(changes: ChangesStream) -> Self {
        let timeout = BATCH_TIMEOUT;
        let max_len = BATCH_MAX_LEN;
        let changes = changes.chunks_timeout(max_len, timeout);
        Self {
            changes,
            last_seq: None,
            typ: PhantomData,
        }
    }

    pub fn last_seq(&self) -> Option<&str> {
        self.last_seq.as_deref()
    }

    fn poll_next(&mut self, cx: &mut Context<'_>) -> Poll<Option<Vec<Record<T>>>> {
        let changes = ready!(self.changes.poll_next_unpin(cx));
        // let changes = ready!(&mut self.changes.poll_next_unpin(cx));
        match changes {
            None => Poll::Ready(None),
            Some(changes) => {
                let last_seq = get_last_seq(&changes[..]);
                let records: Vec<_> = changes
                    .into_iter()
                    .filter_map(|ev| {
                        ev.ok()
                            .and_then(|ev| ev.doc)
                            .and_then(|doc| doc.into_typed_record::<T>().ok())
                    })
                    .collect();
                self.last_seq = last_seq;
                Poll::Ready(Some(records))
            }
        }
    }
}

fn get_last_seq(batch: &[CouchResult<ChangeEvent>]) -> Option<String> {
    batch.last().and_then(|v| match v {
        Ok(v) => Some(v.seq.to_string()),
        _ => None,
    })
}

async fn get_changes(db: CouchDB, params: HashMap<String, String>) -> CouchResult<Response> {
    let req = db.request(Method::GET, "_changes").query(&params);
    let res = req.send().await?;
    Ok(res)
}

impl Stream for ChangesStream {
    type Item = CouchResult<ChangeEvent>;
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            self.state = match self.state {
                ChangesStreamState::Idle => {
                    let mut params = self.params.clone();
                    if let Some(seq) = &self.last_seq {
                        params.insert("since".to_string(), seq.clone());
                    }
                    let fut = get_changes(self.db.clone(), params);
                    ChangesStreamState::Requesting(Box::pin(fut))
                }
                ChangesStreamState::Requesting(ref mut fut) => match ready!(fut.poll_unpin(cx)) {
                    Err(err) => return Poll::Ready(Some(Err(err))),
                    Ok(res) => match res.status().is_success() {
                        true => {
                            let stream = res
                                .bytes_stream()
                                .map_err(|err| io::Error::new(io::ErrorKind::Other, err));
                            let reader = StreamReader::new(stream);
                            let lines = Box::pin(LinesStream::new(reader.lines()));
                            ChangesStreamState::Reading(lines)
                        }
                        false => {
                            return Poll::Ready(Some(Err(CouchError::Couch(
                                res.status(),
                                ErrorDetails::new(
                                    res.status().canonical_reason().unwrap(),
                                    "",
                                    None,
                                ),
                            ))))
                        }
                    },
                },
                ChangesStreamState::Reading(ref mut lines) => {
                    let line = ready!(lines.poll_next_unpin(cx));
                    match line {
                        None => ChangesStreamState::Idle,
                        Some(Err(err)) => {
                            let message = format!("{}", err);
                            let inner = err
                                .into_inner()
                                .and_then(|err| err.downcast::<reqwest::Error>().ok());
                            match inner {
                                Some(reqwest_err) if reqwest_err.is_timeout() && self.infinite => {
                                    ChangesStreamState::Idle
                                }
                                Some(reqwest_err) => {
                                    return Poll::Ready(Some(Err(CouchError::Http(*reqwest_err))));
                                }
                                _ => {
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
                                return Poll::Ready(Some(Ok(event)));
                            }
                            Ok(Event::Finished(event)) => {
                                self.last_seq = Some(event.last_seq.clone());
                                if !self.infinite {
                                    return Poll::Ready(None);
                                }
                                ChangesStreamState::Idle
                            }
                            Err(e) => {
                                return Poll::Ready(Some(Err(e.into())));
                            }
                        },
                    }
                }
            }
        }
    }
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
