use crate::types::{Feed, Media, Post};
use crate::{Record, TypedValue, UntypedRecord};
use std::any::Any;
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct RecordMap {
    records: HashMap<&'static str, HashMap<String, Box<dyn Any + Send>>>,
}

impl RecordMap {
    pub fn from_untyped(records: Vec<UntypedRecord>) -> anyhow::Result<Self> {
        let mut this = RecordMap::default();
        this.insert_untyped_bulk(records)?;
        Ok(this)
    }

    pub fn insert_untyped_bulk(&mut self, records: Vec<UntypedRecord>) -> anyhow::Result<()> {
        for record in records.into_iter() {
            match record.typ() {
                Media::NAME => self.insert_untyped::<Media>(record)?,
                Post::NAME => self.insert_untyped::<Post>(record)?,
                Feed::NAME => self.insert_untyped::<Feed>(record)?,
                _ => {}
            }
        }
        Ok(())
    }

    pub fn insert_untyped<T: TypedValue>(&mut self, record: UntypedRecord) -> anyhow::Result<()> {
        let record = record.into_typed::<T>()?;
        self.insert(record);
        Ok(())
    }

    pub fn insert<T: TypedValue>(&mut self, record: Record<T>) {
        let entry = self
            .records
            .entry(T::NAME)
            .or_insert_with(HashMap::new);
        entry.insert(record.id().to_string(), Box::new(record));
    }

    pub fn get<T: TypedValue>(&self, id: &str) -> Option<&Record<T>> {
        self.records
            .get(T::NAME)
            .and_then(|m| m.get(id))
            .map(|boxed_record| boxed_record.downcast_ref().unwrap())
    }

    pub fn get_all<T: TypedValue>(&self) -> Option<impl Iterator<Item = &Record<T>>> {
        if let Some(records) = self.records.get(T::NAME) {
            let iter = records
                .values()
                .map(|boxed_record| boxed_record.downcast_ref::<Record<T>>().unwrap());
            Some(iter)
        } else {
            None
        }
    }

    pub fn into_iter<T: TypedValue>(&mut self) -> Option<impl Iterator<Item = Record<T>>> {
        if let Some(records) = self.records.remove(T::NAME) {
            let iter = records
                .into_iter()
                .map(|(_, boxed_record)| *boxed_record.downcast::<Record<T>>().unwrap());
            Some(iter)
        } else {
            None
        }
    }

    pub fn into_vec<T: TypedValue>(&mut self) -> Vec<Record<T>> {
        let iter = self.into_iter();
        iter.map(|iter| iter.collect()).unwrap_or_default()
    }

    pub fn into_hashmap<T: TypedValue>(&mut self) -> HashMap<String, Record<T>> {
        self.into_vec::<T>()
            .into_iter()
            .map(|r| (r.guid().to_string(), r))
            .collect()
    }

    pub fn get_mut<T: TypedValue>(&mut self, id: &str) -> Option<&mut Record<T>> {
        self.records
            .get_mut(T::NAME)
            .and_then(|m| m.get_mut(id))
            .map(|boxed_record| boxed_record.downcast_mut().unwrap())
    }
}

// fn upcast_records(records: Vec<UntypedRecord>) -> anyhow::Result<RecordMap> {
//     let mut cache = RecordMap::default();
//     for record in records.into_iter() {
//         match record.typ() {
//             Media::NAME => cache.insert_untyped::<Media>(record)?,
//             Post::NAME => cache.insert_untyped::<Post>(record)?,
//             Feed::NAME => cache.insert_untyped::<Feed>(record)?,
//             _ => {}
//         }
//     }
//     Ok(cache)
// }
