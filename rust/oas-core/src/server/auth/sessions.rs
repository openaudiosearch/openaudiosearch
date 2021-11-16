use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::structs::UserInfo;

/// Session info that's stored for each active session.
#[derive(Debug, Clone)]
pub struct SessionInfo {
    pub(super) is_admin: bool,
    pub(super) user: Arc<UserInfo>,
}

impl SessionInfo {
    pub fn is_admin(&self) -> bool {
        self.is_admin
    }

    pub fn user(&self) -> &Arc<UserInfo> {
        &self.user
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
            .get(session_id).cloned()
    }

    pub async fn remove(&self, session_id: &str) {
        self.sessions.write().await.remove(session_id);
    }
}
