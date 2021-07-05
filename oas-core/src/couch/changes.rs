use futures::io::AsyncBufReadExt;
use futures::io::Lines;
use futures::ready;
use futures::Future;
use futures::FutureExt;
use futures::Stream;
use futures::StreamExt;
use std::collections::HashMap;
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

use super::types::*;
use super::CouchResult;

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
    type Item = CouchResult<ChangeEvent>;
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
