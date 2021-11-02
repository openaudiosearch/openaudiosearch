use std::sync::Arc;

use crate::util::{wait_for_ready, RetryOpts};

use super::{Config, CouchDB, CouchError};

pub const RECORD_DB_NAME: &str = "records";
pub const META_DB_NAME: &str = "meta";
pub const SEPERATOR: &str = "$";

#[derive(Debug, Clone)]
pub struct CouchManager {
    config: Arc<Config>,
    client: reqwest::Client,
    record_db: CouchDB,
    meta_db: CouchDB,
}

pub fn db_name(prefix: &str, name: &str) -> String {
    format!("{}{}{}", prefix, SEPERATOR, name)
}

impl CouchManager {
    pub fn with_url<S>(url: Option<S>) -> anyhow::Result<Self>
    where
        S: AsRef<str>,
    {
        let url = url.map(|s| s.as_ref().to_string());
        let config = Config::from_url_or_default(url.as_deref())?;
        let db = Self::with_config(config)?;
        Ok(db)
    }

    pub fn with_config(config: Config) -> anyhow::Result<Self> {
        let client = reqwest::Client::new();
        let mut record_config = config.clone();
        let mut meta_config = config.clone();
        record_config.database = db_name(&config.database, RECORD_DB_NAME);
        meta_config.database = db_name(&config.database, META_DB_NAME);
        let meta_db = CouchDB::with_config_and_client(meta_config, client.clone());
        let record_db = CouchDB::with_config_and_client(record_config, client.clone());
        Ok(Self {
            config: Arc::new(config),
            client,
            record_db,
            meta_db,
        })
    }

    fn db(&self, name: &str, include_prefix: bool) -> CouchDB {
        let mut config = (*self.config).clone();
        config.database = match include_prefix {
            true => db_name(&config.database, name),
            false => name.to_string(),
        };
        CouchDB::with_config_and_client(config, self.client.clone())
    }

    pub async fn wait_for_ready(&self) -> Result<(), CouchError> {
        let opts = RetryOpts::with_name("CouchDB".into());
        wait_for_ready(&self.client, opts, || {
            self.client.get(&self.config.host).build()
        })
        .await?;
        Ok(())
    }

    /// Test connection and create initial databases.
    pub async fn init(&self) -> anyhow::Result<()> {
        // Wait until the CouchDB is reachable.
        self.wait_for_ready().await?;
        // Init system databases if they do not exist yet.
        let res = futures::future::join_all(vec![
            self.db("_users", false).init(),
            self.db("_replicator", false).init(),
            self.db("_global_changes", false).init(),
        ])
        .await;
        for err in res.into_iter().filter_map(|r| r.err()) {
            log::warn!("Failed to ensure system CouchDB: {}", err);
        }

        let res =
            futures::future::join_all(vec![self.record_db().init(), self.meta_db().init()]).await;
        for res in res {
            res?
        }
        Ok(())
    }

    pub async fn destroy_and_init(&self) -> anyhow::Result<()> {
        let res = futures::future::join_all(vec![
            self.record_db().destroy_and_init(),
            self.meta_db().destroy_and_init(),
        ])
        .await;
        for res in res {
            res?
        }
        Ok(())
    }

    pub fn record_db(&self) -> &CouchDB {
        &self.record_db
    }

    pub fn meta_db(&self) -> &CouchDB {
        &self.meta_db
    }
}
