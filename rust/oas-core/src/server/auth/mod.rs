use rocket::http::{Cookie, CookieJar, Status};
use rocket::request::{FromRequest, Outcome, Request};
use rocket::serde::json::Json;
use rocket::State;
use rocket::{catch, get, post};
use rocket_okapi::openapi;
use std::sync::Arc;
use time::{Duration, OffsetDateTime};
// use okapi::openapi3::Responses;
// use rocket::response::Responder;
// use rocket_okapi::gen::OpenApiGenerator;
// use rocket_okapi::response::OpenApiResponderInner;
// use rocket_okapi::util::add_schema_response;
// use schemars::JsonSchema;
// use serde::Serialize;

mod password;
mod sessions;
mod store;
mod structs;

pub use sessions::{SessionInfo, Sessions};
use store::UserStore;
use structs::{LoginRequest, LoginResponse, RegisterRequest};

use super::error::AppError;

pub const SESSION_COOKIE: &str = "oas_session_id";
pub const SESSION_HEADER: &str = "X-Oas-Session-Id";
pub const SESSION_EXPIRATION: Duration = time::Duration::weeks(8);

#[derive(Debug, Clone)]
pub enum LoginError {
    Unauthorized,
}

/// A session id is a string that is either taken from a private cookie "oas_session_id" or from a
/// header "X-Oas-Session-Id".
#[derive(Debug, PartialEq, Clone)]
pub struct SessionId(String);

async fn try_login(
    auth: &State<Auth>,
    cookies: &CookieJar<'_>,
    login_request: &LoginRequest,
    session_id: Option<&SessionId>,
) -> Option<SessionId> {
    try_logout(&auth, &cookies, session_id).await;
    if let Some(session_id) = auth.login(&login_request).await {
        let cookie = {
            let mut cookie = Cookie::new(SESSION_COOKIE, session_id.clone());
            cookie.set_secure(true);
            cookie.set_http_only(true);
            let mut expiration = OffsetDateTime::now_utc();
            expiration += SESSION_EXPIRATION;
            cookie.set_expires(expiration);
            cookie
        };
        cookies.add(cookie);
        Some(SessionId(session_id))
    } else {
        None
    }
}

async fn try_logout(auth: &State<Auth>, cookies: &CookieJar<'_>, session_id: Option<&SessionId>) {
    if let Some(session_id) = session_id {
        auth.logout(&session_id.0).await;
    }

    if let Some(session_cookie) = cookies.get(SESSION_COOKIE) {
        auth.logout(session_cookie.value()).await;
        cookies.remove(session_cookie.clone());
    }
}

#[async_trait::async_trait]
impl<'r> FromRequest<'r> for SessionId {
    type Error = LoginError;
    async fn from_request(request: &'r Request<'_>) -> Outcome<SessionId, LoginError> {
        let auth = request
            .guard::<&State<Auth>>()
            .await
            .expect("Session state not registered");

        // Check if a session id is set as a cookie.
        let session_cookie = request.cookies().get(SESSION_COOKIE);
        let session_id = session_cookie.map(|cookie| SessionId(cookie.value().to_string()));

        // Check if a session id is set as a header.
        let session_header = request.headers().get_one(SESSION_HEADER);
        let session_id = if let Some(session_header) = session_header {
            Some(SessionId(session_header.to_string()))
        } else {
            session_id
        };

        // Allow to login via basic auth.
        // If using basic auth while also having a session id set,
        // first delete the session.
        let session_id = if session_id.is_none() {
            let basic_auth = request.guard::<BasicAuth>().await;
            if let Outcome::Success(basic_auth) = basic_auth {
                try_login(
                    &auth,
                    request.cookies(),
                    &basic_auth.into(),
                    session_id.as_ref(),
                )
                .await
            } else {
                None
            }
        } else {
            session_id
        };

        match session_id {
            Some(id) => Outcome::Success(id),
            None => Outcome::Forward(()),
        }
    }
}

#[async_trait::async_trait]
impl<'r> FromRequest<'r> for SessionInfo {
    type Error = LoginError;
    async fn from_request(request: &'r Request<'_>) -> Outcome<SessionInfo, LoginError> {
        let auth = request
            .guard::<&State<Auth>>()
            .await
            .expect("Session state not registered");

        let session_id = request.guard::<SessionId>().await;

        match session_id {
            Outcome::Success(session_id) => {
                if let Some(session) = auth.sessions.get(&session_id.0).await {
                    Outcome::Success((*session).clone())
                } else {
                    Outcome::Failure((Status::Unauthorized, LoginError::Unauthorized))
                }
            }
            _ => Outcome::Failure((Status::Unauthorized, LoginError::Unauthorized)),
        }
    }
}

/// Request guard marker struct for logged in users with "is_admin == true".
#[derive(Debug, Clone)]
pub struct AdminUser {}

#[async_trait::async_trait]
impl<'r> FromRequest<'r> for AdminUser {
    type Error = LoginError;
    async fn from_request(request: &'r Request<'_>) -> Outcome<AdminUser, LoginError> {
        let session = request.guard::<SessionInfo>().await;
        match session {
            Outcome::Success(session) => {
                if session.is_admin() {
                    Outcome::Success(AdminUser {})
                } else {
                    Outcome::Failure((Status::Unauthorized, LoginError::Unauthorized))
                }
            }
            Outcome::Failure(x) => Outcome::Failure(x),
            Outcome::Forward(x) => Outcome::Forward(x),
        }
    }
}

/// The auth state contains user and session stores.
#[derive(Debug, Clone)]
pub struct Auth {
    pub(crate) sessions: Sessions,
    pub(crate) users: UserStore,
}

impl Auth {
    /// Create a new, empty auth state.
    pub fn new() -> Self {
        Self {
            sessions: Sessions::new(),
            users: UserStore::new(),
        }
    }

    pub async fn ensure_admin_user(&self, password: &str) {
        self.users.add_admin_user(password).await;
    }

    /// Try to login with username and password. Returns a new, random session ID in case of
    /// success.
    pub async fn login(&self, req: &LoginRequest) -> Option<String> {
        let user = self.users.login(&req).await;
        if let Some(user) = user {
            // let user_info = UserInfo {
            //     username: username.to_string(),
            // };
            let session_id = generate_session_id();
            // let user = AdminUser {};
            let info = SessionInfo {
                is_admin: true,
                user: Arc::new(user),
            };
            self.sessions.insert(&session_id, info).await;
            Some(session_id)
        } else {
            None
        }
    }

    /// Get the session info for an active session.
    pub async fn session(&self, session_id: &str) -> Option<Arc<SessionInfo>> {
        self.sessions.get(session_id).await
    }

    /// Logout a user by session id.
    pub async fn logout(&self, session_id: &str) {
        self.sessions.remove(&session_id).await;
    }
}

#[openapi(tag = "Login")]
#[get("/login")]
pub async fn get_login(session: Option<SessionInfo>) -> Json<LoginResponse> {
    match session {
        Some(session) => Json(LoginResponse {
            ok: true,
            user: Some(session.user().into_public()),
        }),
        None => Json(LoginResponse {
            ok: false,
            user: None,
        }),
    }
}

#[openapi(tag = "Login")]
#[post("/login", data = "<data>")]
pub async fn post_login(
    auth: &State<Auth>,
    cookies: &CookieJar<'_>,
    data: Json<LoginRequest>,
    session_id: Option<SessionId>,
) -> Json<LoginResponse> {
    let session_id = try_login(&auth, &cookies, &data, session_id.as_ref()).await;
    if let Some(session_id) = session_id {
        let session = auth.session(&session_id.0).await.unwrap();
        let public_user_info = session.user().into_public();
        Json(LoginResponse {
            ok: true,
            user: Some(public_user_info),
        })
    } else {
        Json(LoginResponse {
            ok: false,
            user: None,
        })
    }
}

#[openapi(tag = "Login")]
#[post("/logout")]
pub async fn logout(
    auth: &State<Auth>,
    cookies: &CookieJar<'_>,
    session_id: Option<SessionId>,
) -> Json<()> {
    try_logout(&auth, &cookies, session_id.as_ref()).await;
    Json(())
}

#[openapi(tag = "Login")]
#[post("/register", data = "<user>")]
pub async fn register(
    _is_admin: AdminUser,
    auth: &State<Auth>,
    user: Json<RegisterRequest>,
) -> Json<()> {
    auth.users.register(user.into_inner(), false).await;
    Json(())
}

#[openapi(tag = "Login")]
#[get("/private")]
pub async fn private(_user: AdminUser) -> String {
    format!("you're logged in!")
}

#[catch(401)]
pub fn unauthorized(_req: &rocket::Request) -> AppError {
    AppError::Unauthorized
}

// /// Helper enum to implement an okapi responder.
// /// This only sets an Unauthorized status code for the second variant.
//
// This is likely not needed anymore because the login route now always returns 200.
// pub enum LoginResult<T> {
//     Ok(Json<T>),
//     Unauthorized(Json<T>),
// }

// impl<T> LoginResult<T> {
//     pub fn into_inner(self) -> Json<T> {
//         match self {
//             Self::Ok(value) => value,
//             Self::Unauthorized(value) => value,
//         }
//     }
// }

// impl<'r, T> Responder<'r, 'static> for LoginResult<T>
// where
//     T: Serialize,
// {
//     fn respond_to(self, req: &'r Request<'_>) -> rocket::response::Result<'static> {
//         let (status, header) = match self {
//             Self::Ok(_) => (Status::Ok, None),
//             Self::Unauthorized(_) => {
//                 let header_value = format!(
//                     r#"Basic realm="{}", charset="UTF-8""#,
//                     "Please enter user username and password"
//                 );
//                 let header = Header::new(http::header::WWW_AUTHENTICATE.as_str(), header_value);
//                 (Status::Unauthorized, Some(header))
//             }
//         };
//         let mut res = self.into_inner().respond_to(&req)?;
//         if let Some(header) = header {
//             res.set_header(header);
//         }
//         res.set_status(status);
//         Ok(res)
//     }
// }

// impl<T: Serialize + JsonSchema + Send> OpenApiResponderInner for LoginResult<T> {
//     fn responses(gen: &mut OpenApiGenerator) -> rocket_okapi::Result<Responses> {
//         let mut responses = Responses::default();
//         let schema = gen.json_schema::<T>();
//         add_schema_response(&mut responses, 200, "application/json", schema.clone())?;
//         add_schema_response(&mut responses, 401, "application/json", schema)?;
//         Ok(responses)
//     }
// }

/// Generate a random session id.
fn generate_session_id() -> String {
    let random_bytes: [u8; 32] = rand::random();
    let encoded = base32::encode(base32::Alphabet::Crockford, &random_bytes[..]);
    encoded.to_lowercase()
}

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

    let split_vec: Vec<&str> = decoded_creds.splitn(2, ":").collect();

    if split_vec.len() < 2 {
        None
    } else {
        Some((split_vec[0].to_string(), split_vec[1].to_string()))
    }
}
