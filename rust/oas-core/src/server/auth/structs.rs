use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema, Debug, PartialEq, Clone)]
pub struct RegisterRequest {
    pub password: String,
    pub username: String,
    pub email: Option<String>,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, PartialEq, Clone)]
pub struct UserPublicInfo {
    pub username: String,
    pub email: Option<String>,
    pub is_admin: bool,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, PartialEq, Clone)]
pub struct UserInfo {
    pub password: String,
    pub username: String,
    pub is_admin: bool,
    pub email: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize, JsonSchema)]
pub struct LoginResponse {
    pub ok: bool,
    pub user: Option<UserPublicInfo>,
}

impl UserInfo {
    pub fn into_public(&self) -> UserPublicInfo {
        self.clone().into()
    }
}

impl From<UserInfo> for UserPublicInfo {
    fn from(user: UserInfo) -> Self {
        Self {
            username: user.username,
            email: user.email,
            is_admin: user.is_admin,
        }
    }
}
