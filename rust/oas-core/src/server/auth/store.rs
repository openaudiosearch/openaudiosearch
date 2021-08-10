use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::password;
use super::structs::{LoginRequest, RegisterRequest, UserInfo};

#[derive(Debug, Clone)]
pub struct UserStore {
    users: Arc<RwLock<HashMap<String, UserInfo>>>,
}

impl UserStore {
    pub fn new() -> Self {
        Self {
            users: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn _with_admin_user(password: &str) -> Self {
        let this = Self::new();
        this.add_admin_user(&password).await;
        this
    }

    pub async fn add_admin_user(&self, password: &str) {
        let req = RegisterRequest {
            username: "admin".to_string(),
            password: password.to_string(),
            email: None,
        };
        self.register(req, true).await
    }

    pub async fn register(&self, req: RegisterRequest, is_admin: bool) {
        let password = password::hash_password(&req.password);
        let username = req.username;
        let user = UserInfo {
            username,
            password,
            email: req.email,
            is_admin,
        };
        self.users
            .write()
            .await
            .insert(user.username.to_string(), user);
    }

    pub async fn login(&self, req: &LoginRequest) -> Option<UserInfo> {
        self.verify_and_get(&req.username, &req.password).await
    }

    pub async fn verify_and_get(&self, username: &str, password: &str) -> Option<UserInfo> {
        let store = self.users.read().await;
        let user = store.get(username);
        if let Some(user) = user {
            if password::verify_password(&user.password, password) {
                Some((*user).clone())
            } else {
                None
            }
        } else {
            None
        }
    }
}
