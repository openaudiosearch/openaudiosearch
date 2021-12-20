enum Message {
    Publish(PublishOp),
    Query(QueryOp),
    Job(JobOp)
}

struct PublishOp {
    puts: Vec<Record>,
    dels: Vec<Address>,
}

struct Address {
    origin: Option<String>,
    typ: String,
    id: Uuid
}
