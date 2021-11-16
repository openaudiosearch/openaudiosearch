use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use lazy_static::lazy_static;
use rand_core::OsRng;

lazy_static! {
    static ref SALT: SaltString = {
        
        SaltString::generate(&mut OsRng)
    };
}

lazy_static! {
    static ref ARGON2: Argon2<'static> = Argon2::default();
}

pub fn hash_password(password: &str) -> String {
    ARGON2
        .hash_password_simple(password.as_bytes(), SALT.as_ref())
        .unwrap()
        .to_string()
}

pub fn verify_password(password_hash: &str, password: &str) -> bool {
    let parsed_hash = PasswordHash::new(password_hash).unwrap();
    ARGON2
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
}
