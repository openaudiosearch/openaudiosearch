use crate::types::Media;
use crate::{Record, TypedValue};

pub fn debug_print_records<T>(records: &[Record<T>])
where
    T: TypedValue,
{
    for record in records {
        debug_print_record(record)
    }
}
pub fn debug_print_record<T>(record: &Record<T>)
where
    T: TypedValue,
{
    eprintln!(
        r#"<Record {}_{} [{}]>"#,
        record.id(),
        record.typ(),
        record.value.label().unwrap_or_default()
    );
}
