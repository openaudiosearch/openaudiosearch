use okapi::openapi3::Responses;
use rocket::http::{Cookie, CookieJar, Status};
use rocket::request::{FromRequest, Outcome, Request};
use rocket::response::Responder;
use rocket::serde::json::Json;
use rocket::State;
use rocket::{get, post};
use rocket_okapi::gen::OpenApiGenerator;
use rocket_okapi::openapi;
use rocket_okapi::response::OpenApiResponderInner;
use rocket_okapi::util::add_schema_response;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

pub const COOKIE_SESSION_ID: &str = "oas_session_id";
pub const HEADER_SESSION_ID: &str = "X-Oas-Session-Id";

#[derive(Debug, Clone)]
pub enum LoginError {
    Unauthorized,
}

/// A session id is a string that is either taken from a private cookie "oas_session_id" or from a
/// header "X-Oas-Session-Id".
pub struct SessionId(String);

#[async_trait::async_trait]
impl<'r> FromRequest<'r> for SessionId {
    type Error = LoginError;
    async fn from_request(request: &'r Request<'_>) -> Outcome<SessionId, LoginError> {
        // Check cookie auth
        let mut session_id = request
            .cookies()
            .get_private(COOKIE_SESSION_ID)
            .map(|cookie| cookie.value().to_string());

        // Check header auth.
        if session_id.is_none() {
            session_id = request
                .headers()
                .get_one(HEADER_SESSION_ID)
                .map(|v| v.to_string());
        }
        match session_id {
            Some(id) => Outcome::Success(SessionId(id)),
            None => Outcome::Forward(()),
        }
    }
}

/// Session info that's stored for each active session.
#[derive(Debug, Clone)]
pub struct SessionInfo {
    is_admin: bool,
    user: Arc<UserInfo>,
}

/// User info for active sessions.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UserInfo {
    username: String,
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

        if let Outcome::Success(session_id) = session_id {
            if let Some(session) = auth.sessions.get(&session_id.0).await {
                Outcome::Success((*session).clone())
            } else {
                Outcome::Failure((Status::Unauthorized, LoginError::Unauthorized))
            }
        } else {
            Outcome::Failure((Status::Unauthorized, LoginError::Unauthorized))
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
                if session.is_admin {
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

/// This is a very simple session store. It does not yet feature expiration.
/// TODO: Add expiration.
/// TODO: Use redis.
#[derive(Debug, Clone)]
pub struct Sessions {
    sessions: Arc<RwLock<HashMap<String, Arc<SessionInfo>>>>,
}

impl Sessions {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn insert(&self, id: &str, info: SessionInfo) {
        self.sessions
            .write()
            .await
            .insert(id.to_string(), Arc::new(info));
    }

    pub async fn get(&self, session_id: &str) -> Option<Arc<SessionInfo>> {
        self.sessions
            .read()
            .await
            .get(session_id)
            .map(|session_info| session_info.clone())
    }

    pub async fn remove(&self, session_id: &str) {
        self.sessions.write().await.remove(session_id);
    }
}

/// User store.
/// It does not acutally store any users.
/// TODO: Decide where users are stored.
/// TODO: Decide which password hashing to use.
#[derive(Debug, Clone)]
pub struct UserStore {
    users: Arc<RwLock<HashMap<String, String>>>,
}

impl UserStore {
    pub fn new() -> Self {
        let mut users = HashMap::new();
        // TODO: Implement actual users store ;)
        users.insert("admin".to_string(), "password".to_string());
        Self {
            users: Arc::new(RwLock::new(users)),
        }
    }

    pub async fn check_login(&self, username: &str, password: &str) -> bool {
        match self.users.read().await.get(username) {
            Some(registered_password) => registered_password.as_str() == password,
            None => false,
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

    /// Try to login with username and password. Returns a new, random session ID in case of
    /// success.
    pub async fn login(&self, username: &str, password: &str) -> Option<String> {
        let is_ok = self.users.check_login(&username, &password).await;
        if is_ok {
            let user_info = UserInfo {
                username: username.to_string(),
            };
            let session_id = generate_session_id();
            // let user = AdminUser {};
            let info = SessionInfo {
                is_admin: true,
                user: Arc::new(user_info),
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

#[derive(Deserialize, JsonSchema)]
pub struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Serialize, JsonSchema)]
pub struct LoginResponse {
    ok: bool,
    user: Option<UserInfo>,
}

#[openapi(tag = "Login")]
#[get("/login")]
pub async fn get_login(session: Option<SessionInfo>) -> LoginResult<LoginResponse> {
    match session {
        Some(session) => LoginResult::Ok(Json(LoginResponse {
            ok: true,
            user: Some((*session.user).clone()),
        })),
        None => LoginResult::Unauthorized(Json(LoginResponse {
            ok: false,
            user: None,
        })),
    }
}

#[openapi(tag = "Login")]
#[post("/login", data = "<value>")]
pub async fn post_login(
    auth: &State<Auth>,
    cookies: &CookieJar<'_>,
    value: Json<LoginRequest>,
) -> LoginResult<LoginResponse> {
    let session_id = auth.login(&value.username, &value.password).await;
    if let Some(session_id) = session_id {
        // Unwrap is save because the session was just created.
        let session = auth.session(&session_id).await.unwrap();
        cookies.add_private(Cookie::new(COOKIE_SESSION_ID, session_id));
        LoginResult::Ok(Json(LoginResponse {
            ok: true,
            user: Some((*session.user).clone()),
        }))
    } else {
        LoginResult::Unauthorized(Json(LoginResponse {
            ok: false,
            user: None,
        }))
    }
}

#[openapi(tag = "Login")]
#[post("/logout")]
pub async fn logout(auth: &State<Auth>, cookies: &CookieJar<'_>) -> Json<()> {
    if let Some(session_cookie) = cookies.get_private(COOKIE_SESSION_ID) {
        auth.logout(session_cookie.value()).await;
        cookies.remove_private(session_cookie);
    }
    Json(())
}

#[openapi(tag = "Login")]
#[get("/private")]
pub async fn private(_user: AdminUser) -> String {
    format!("you're logged in!")
}

/// Helper enum to implement an okapi responder.
/// This only sets an Unauthorized status code for the second variant.
pub enum LoginResult<T> {
    Ok(Json<T>),
    Unauthorized(Json<T>),
}

impl<T> LoginResult<T> {
    pub fn into_inner(self) -> Json<T> {
        match self {
            Self::Ok(value) => value,
            Self::Unauthorized(value) => value,
        }
    }
}

impl<'r, T> Responder<'r, 'static> for LoginResult<T>
where
    T: Serialize,
{
    fn respond_to(self, req: &'r Request<'_>) -> rocket::response::Result<'static> {
        self.into_inner().respond_to(&req)
    }
}

impl<T: Serialize + JsonSchema + Send> OpenApiResponderInner for LoginResult<T> {
    fn responses(gen: &mut OpenApiGenerator) -> rocket_okapi::Result<Responses> {
        let mut responses = Responses::default();
        let schema = gen.json_schema::<T>();
        add_schema_response(&mut responses, 200, "application/json", schema.clone())?;
        add_schema_response(&mut responses, 401, "application/json", schema)?;
        Ok(responses)
    }
}

/// Generate a random session id
pub fn generate_session_id() -> String {
    let uuid = Uuid::new_v4();
    let encoded = base32::encode(base32::Alphabet::Crockford, &uuid.as_bytes()[..]);
    encoded.to_lowercase()
}
