use crate::types::AudioObject;
use crate::Record;

pub fn debug_print_records(records: &[Record<AudioObject>]) {
    for record in records {
        debug_print_record(record)
    }
}
pub fn debug_print_record(record: &Record<AudioObject>) {
    eprintln!(
        r#"<Record {} [{}] "{}">"#,
        record.id(),
        record.value.identifier.as_deref().unwrap_or("missing_id"),
        record.value.headline.as_deref().unwrap_or("")
    );
}
