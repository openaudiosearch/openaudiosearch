use crate::types::Media;
use crate::Record;

pub fn debug_print_records(records: &[Record<Media>]) {
    for record in records {
        debug_print_record(record)
    }
}
pub fn debug_print_record(record: &Record<Media>) {
    eprintln!(
        r#"<Record {} [{}] "{}">"#,
        record.id(),
        record.value.identifier.as_deref().unwrap_or("missing_id"),
        record.value.headline.as_deref().unwrap_or("")
    );
}
