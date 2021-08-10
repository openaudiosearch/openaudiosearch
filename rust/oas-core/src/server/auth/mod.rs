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
use serde::Serialize;
use std::sync::Arc;
use uuid::Uuid;

mod password;
mod sessions;
mod store;
mod structs;

pub use sessions::{SessionInfo, Sessions};
use store::UserStore;
use structs::{LoginRequest, LoginResponse, RegisterRequest};

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
pub async fn get_login(session: Option<SessionInfo>) -> LoginResult<LoginResponse> {
    match session {
        Some(session) => LoginResult::Ok(Json(LoginResponse {
            ok: true,
            user: Some(session.user().into_public()),
        })),
        None => LoginResult::Unauthorized(Json(LoginResponse {
            ok: false,
            user: None,
        })),
    }
}

#[openapi(tag = "Login")]
#[post("/login", data = "<data>")]
pub async fn post_login(
    auth: &State<Auth>,
    cookies: &CookieJar<'_>,
    data: Json<LoginRequest>,
) -> LoginResult<LoginResponse> {
    let session_id = auth.login(&data).await;
    if let Some(session_id) = session_id {
        // Unwrap is save because the session was just created.
        let session = auth.session(&session_id).await.unwrap();
        cookies.add_private(Cookie::new(COOKIE_SESSION_ID, session_id));
        let public_user_info = session.user().into_public();
        LoginResult::Ok(Json(LoginResponse {
            ok: true,
            user: Some(public_user_info),
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
        let status = match self {
            Self::Ok(_) => Status::Ok,
            Self::Unauthorized(_) => Status::Unauthorized,
        };
        let mut res = self.into_inner().respond_to(&req)?;
        res.set_status(status);
        Ok(res)
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
