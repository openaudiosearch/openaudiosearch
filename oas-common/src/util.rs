use sha2::Digest;
use uuid::Uuid;

use crate::{DecodingError, TypedValue};

pub fn split_guid(guid: &str) -> Option<(String, String)> {
    let split: Vec<&str> = guid.split("_").collect();
    if split.len() != 2 {
        None
    } else {
        Some((split[0].to_string(), split[1].to_string()))
    }
}

pub fn split_and_check_guid<T: TypedValue>(guid: &str) -> Result<(String, String), DecodingError> {
    let split = split_guid(guid);
    if let Some((typ, id)) = split {
        if typ == T::NAME {
            Ok((typ, id))
        } else {
            Err(DecodingError::TypeMismatch(typ, T::NAME.to_string()))
        }
    } else {
        Ok((T::NAME.to_string(), guid.to_string()))
    }
}

pub fn id_from_hashed_string(string: impl AsRef<str>) -> String {
    let mut hasher = sha2::Sha256::new();
    hasher.update(string.as_ref().as_bytes());
    let result = hasher.finalize();
    let encoded = base32::encode(base32::Alphabet::Crockford, &result[0..16]);
    encoded.to_lowercase()
}

pub fn id_from_uuid() -> String {
    let uuid = Uuid::new_v4();
    let encoded = base32::encode(base32::Alphabet::Crockford, &uuid.as_bytes()[0..16]);
    encoded.to_lowercase()
}
