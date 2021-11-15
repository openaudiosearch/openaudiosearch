use rocket::http::Status;
use rocket::request::{FromRequest, Outcome, Request};

use super::structs::LoginRequest;

// basic_auth module originally taken from https://github.com/Owez/rocket-basicauth
// Copyright © 2020 Owen Griffiths
// License: MIT
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is furnished
// to do so, subject to the following conditions:
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED,
// INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
// PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT
// HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF
// CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE
// OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

#[derive(Debug)]
pub struct BasicAuth {
    /// Required username
    pub username: String,

    /// Required password
    pub password: String,
}

impl BasicAuth {
    /// Creates a new [BasicAuth] struct/request guard from a given plaintext
    /// http auth header or returns a [Option::None] if invalid
    pub fn new<T: Into<String>>(auth_header: T) -> Option<Self> {
        let key = auth_header.into();

        if key.len() < 7 || &key[..6] != "Basic " {
            return None;
        }

        let (username, password) = decode_basic_auth(&key[6..])?;

        Some(Self { username, password })
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for BasicAuth {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let keys: Vec<_> = request.headers().get("Authorization").collect();
        match keys.len() {
            0 => Outcome::Forward(()),
            1 => match BasicAuth::new(keys[0]) {
                Some(auth_header) => Outcome::Success(auth_header),
                None => Outcome::Failure((Status::BadRequest, ())),
            },
            _ => Outcome::Failure((Status::BadRequest, ())),
        }
    }
}

impl From<BasicAuth> for LoginRequest {
    fn from(auth: BasicAuth) -> Self {
        Self {
            username: auth.username,
            password: auth.password,
        }
    }
}

/// Decodes a base64-encoded string into a tuple of `(username, password)` or a
/// [Option::None] if badly formatted, e.g. if an error occurs
fn decode_basic_auth<T: Into<String>>(base64_encoded: T) -> Option<(String, String)> {
    let decoded_creds = match base64::decode(base64_encoded.into()) {
        Ok(vecu8_creds) => String::from_utf8(vecu8_creds).unwrap(),
        Err(_) => return None,
    };

    let split_vec: Vec<&str> = decoded_creds.splitn(2, ':').collect();

    if split_vec.len() < 2 {
        None
    } else {
        Some((split_vec[0].to_string(), split_vec[1].to_string()))
    }
}
