
use crate::couch::{CouchDB};

use async_std::stream::StreamExt;

use oas_common::types;
use oas_common::TypedRecord;
use oas_common::TypedValue;
use std::collections::HashMap;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::spawn;


#[derive(Debug)]
pub struct FeedManager {
    store: FeedStore,
}

#[derive(Clone, Debug)]
pub struct FeedStore {
    data: Arc<Mutex<HashMap<String, TypedRecord<types::Feed>>>>,
    sender: Sender<(String, TypedRecord<types::Feed>)>,
}

impl FeedStore {
    pub fn new(
        sender: Sender<(String, TypedRecord<types::Feed>)>) -> Self {
        Self {
            data: Arc::new(Mutex::new(HashMap::new())),
            sender: sender,
        }
    }
}

impl FeedManager {
    pub fn new() -> Self {
        let (sender, receiver) = channel();
        let manager = Self {
            store: FeedStore::new(sender),
        };
        return manager
        
    }
    pub fn print(&self) {
        eprintln!("FeedStore: {:?}", self.store)
    }

    pub async fn init(&'static mut self, db: &CouchDB) -> anyhow::Result<()> {
        let records = db.get_all_records::<types::Feed>().await?;
        for record in records {
            self.store.sender.send((record.id().into(), record))?;
        }
        Ok(())
    }
    
}
fn run(manager: FeedManager, receiver : Receiver<(String, TypedRecord<types::Feed>)> ) -> anyhow::Result<()> {
    let data = manager.store.data.clone();
    eprintln!("RUN METHOD");
    spawn(move || {
        for i in receiver {
            let (id, value) = i;
            let mut data = data.lock().unwrap();
            data.insert(id, value);
        }
    });

    Ok(())
}

/// Checks the CouchDB [ChangesStream] for incoming feed records.
pub async fn watch_changes(manager: &mut FeedManager, db: &CouchDB) -> anyhow::Result<()> {
    eprintln!("WATCHCHENGES");
    let mut stream = db.changes(None);
    stream.set_infinite(true);
    while let Some(event) = stream.next().await {
        let event = event?;
        if let Some(doc) = event.doc {
            let _id = doc.id().to_string();
            let record = doc.into_typed_record::<types::Feed>();
            match record {
                Err(_err) => {}
                Ok(record) => match record.typ() {
                    types::Feed::NAME => {
                        let url = &record.value.url;
                        manager.store.sender.send((url.clone(), record.clone()))?;
                    }
                    _ => {}
                },
            }
        }
    }
    Ok(())
}

// pub async fn feed_task_loop(db: &CouchDB, feed: Feed) -> Result<(), RssError> {
//     loop {
//         feed.load().await?;
//         let items = feed.into_medias()?;
//         let docs: Vec<Doc> = items.iter().map(|r| r.clone().into()).collect();
//         let res = db.put_bulk(docs).await?;
//     }
// }
