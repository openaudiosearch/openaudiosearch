struct RecordStore {
    store: HashMap<TypeId, HashMap<String, Box<dyn Any>>>,
    blank: HashMap<String, Record>,
}

impl RecordStore {
    pub fn new() -> Self {
        Self {
            store: HashMap::new(),
            blank: HashMap::new(),
        }
    }
    pub fn insert_blank(&mut self, record: Record) {
        self.blank.insert(record.id().clone(), record);
    }

    pub fn insert<T: RecordValue>(&mut self, record: Record) {
        let typeid = TypeId::of::<T>();
        let id = record.id().clone();
        let record: Box<dyn Any> = Box::new(record);
        let entry = self.store.entry(typeid).or_default();
        entry.insert(id, record);
    }

    pub fn get<T: RecordValue>(&self, id: &str) -> Option<&Record> {
        let typeid = TypeId::of::<T>();
        let record = self
            .store
            .get(&typeid)
            .and_then(|bucket| bucket.get(id))
            .and_then(|record| record.downcast_ref::<Record>());
        record
        // if record.is_none() {
        //     let record = self
        //         .blank
        //         .get(id)
        //         .and_then(|blank| blank.clone().upcast::<T>().ok());
        //     record
        // } else {
        //     record
        // }
    }
    // fn get<T: RecordValue)(&self, id: &str) -> Result<&Record<T>, UpcastError> {
    // }
}
