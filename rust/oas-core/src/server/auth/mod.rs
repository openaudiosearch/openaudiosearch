use rocket::http::{Cookie, CookieJar, Status};
use rocket::request::{FromRequest, Outcome, Request};
use rocket::serde::json::Json;
use rocket::State;
use rocket::{get, post};
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

pub const COOKIE_SESSION_ID: &str = "oas_session_id";

#[derive(Debug, Clone)]
pub struct AdminUserInfo {
    username: String,
    password: String,
}

pub fn registered_users() -> HashMap<String, String> {
    let mut map = HashMap::new();
    map.insert("admin".to_string(), "password".to_string());
    map
}

#[derive(Debug, Clone)]
pub struct UserStore {
    users: Arc<RwLock<HashMap<String, String>>>,
}

impl UserStore {
    pub fn new() -> Self {
        let users = registered_users();
        Self {
            users: Arc::new(RwLock::new(users)),
        }
    }

    pub async fn check_login(&self, username: &str, password: &str) -> bool {
        // let registered_password = self.users.read().await.get(username);
        match self.users.read().await.get(username) {
            Some(registered_password) => registered_password.as_str() == password,
            None => false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AdminUser {}

#[derive(Debug, Clone)]
pub enum LoginError {
    Unauthorized,
}

#[async_trait::async_trait]
impl<'r> FromRequest<'r> for AdminUser {
    type Error = LoginError;
    async fn from_request(request: &'r Request<'_>) -> Outcome<AdminUser, LoginError> {
        let auth = request
            .guard::<&State<Auth>>()
            .await
            .expect("Session state not registered");

        // Check cookie auth
        let mut session_id = request
            .cookies()
            .get_private(COOKIE_SESSION_ID)
            .map(|cookie| cookie.value().to_string());
        // Check header auth.
        if session_id.is_none() {
            session_id = request
                .headers()
                .get_one("X-OAS-Session-Id")
                .map(|v| v.to_string());
        }
        if let Some(session_id) = session_id {
            if let Some(session) = auth.sessions.get(&session_id).await {
                let admin_user = session.user.clone();
                Outcome::Success((*admin_user).clone())
            } else {
                Outcome::Failure((Status::Unauthorized, LoginError::Unauthorized))
            }
        } else {
            Outcome::Failure((Status::Unauthorized, LoginError::Unauthorized))
        }
    }
}

#[derive(Debug, Clone)]
pub struct SessionInfo {
    user: Arc<AdminUser>,
}

/// This is a very simple session store. It does not yet feature expiration.
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

#[derive(Debug, Clone)]
pub struct Auth {
    pub(crate) sessions: Sessions,
    pub(crate) users: UserStore,
}

impl Auth {
    pub fn new() -> Self {
        Self {
            sessions: Sessions::new(),
            users: UserStore::new(),
        }
    }

    pub async fn login(&self, username: &str, password: &str) -> Option<String> {
        let is_ok = self.users.check_login(&username, &password).await;
        if is_ok {
            let session_id = generate_session_id();
            let user = AdminUser {};
            let info = SessionInfo {
                user: Arc::new(user),
            };
            self.sessions.insert(&session_id, info).await;
            Some(session_id)
        } else {
            None
        }
    }

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
}

#[openapi(tag = "Login")]
#[post("/login", data = "<value>")]
pub async fn login(
    auth: &State<Auth>,
    cookies: &CookieJar<'_>,
    value: Json<LoginRequest>,
) -> Json<LoginResponse> {
    let maybe_session_id = auth.login(&value.username, &value.password).await;
    if let Some(session_id) = maybe_session_id {
        cookies.add_private(Cookie::new(COOKIE_SESSION_ID, session_id));
        Json(LoginResponse { ok: true })
    } else {
        Json(LoginResponse { ok: false })
    }
}

#[openapi(tag = "Login")]
#[post("/logout")]
pub async fn logout(auth: &State<Auth>, cookies: &CookieJar<'_>) -> Json<()> {
    let cookie = cookies.get_private(COOKIE_SESSION_ID);
    if let Some(cookie) = cookie {
        auth.logout(cookie.value()).await;
    }
    Json(())
}

#[openapi(tag = "Login")]
#[get("/private")]
pub async fn private(_user: AdminUser) -> String {
    format!("you're logged in!")
}

/// Random session id
pub fn generate_session_id() -> String {
    let uuid = Uuid::new_v4();
    let encoded = base32::encode(base32::Alphabet::Crockford, &uuid.as_bytes()[..]);
    encoded.to_lowercase()
}
