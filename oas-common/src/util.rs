use sha2::Digest;
use uuid::Uuid;

pub fn id_from_hashed_string(string: String) -> String {
    let mut hasher = sha2::Sha256::new();
    hasher.update(string.as_bytes());
    let result = hasher.finalize();
    let encoded = base32::encode(base32::Alphabet::Crockford, &result[0..16]);
    encoded.to_lowercase()
}

pub fn id_from_uuid() -> String {
    let uuid = Uuid::new_v4();
    let encoded = base32::encode(base32::Alphabet::Crockford, &uuid.as_bytes()[0..16]);
    encoded.to_lowercase()
}
