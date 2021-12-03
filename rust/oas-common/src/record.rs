use std::convert::TryFrom;

mod error;
mod meta;
mod traits;
mod typed;
mod untyped;

pub use error::{DecodingError, EncodingError, ValidationError};
pub use meta::RecordMeta;
pub use traits::TypedValue;
pub use typed::TypedRecord;
pub use untyped::UntypedRecord;

pub type Object = serde_json::Map<String, serde_json::Value>;
pub type Record<T> = TypedRecord<T>;

impl<T> TryFrom<UntypedRecord> for TypedRecord<T>
where
    T: TypedValue,
{
    type Error = DecodingError;
    fn try_from(record: UntypedRecord) -> Result<Self, Self::Error> {
        record.into_typed_record()
    }
}

impl<T> TryFrom<TypedRecord<T>> for UntypedRecord
where
    T: TypedValue,
{
    type Error = EncodingError;
    fn try_from(record: TypedRecord<T>) -> Result<Self, Self::Error> {
        record.into_untyped()
    }
}
