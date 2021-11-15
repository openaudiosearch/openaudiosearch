use dirs::config_dir;
use serde::Deserialize;
use toml;
extern crate dirs;
use anyhow::Context;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs::{metadata, read_to_string};

pub type AllMappings = HashMap<String, String>;

const DEFAULT_MAPPING: &str = include_str!("../../../../config/mapping.toml");

#[derive(Debug)]
pub struct MappingManager {
    mappings: HashMap<String, Mapping>,
    path: Option<PathBuf>,
}

#[derive(Deserialize, Debug, Default, Clone)]
pub struct Mapping {
    fields: Vec<FieldMapping>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct FieldMapping {
    pub from: String,
    pub to: String,
}

impl MappingManager {
    pub fn new() -> Self {
        Self {
            mappings: Default::default(),
            path: None,
        }
    }

    pub fn with_file(f: &str) -> Self {
        Self {
            mappings: Default::default(),
            path: Some(f.to_string().into()),
        }
    }

    pub fn to_field_hashmap(&self) -> HashMap<String, String> {
        let map: HashMap<String, String> = self
            .mappings
            .clone()
            .into_iter()
            .map(|(_namespace, mapping)| mapping.fields.into_iter())
            .flatten()
            .map(|field| (field.from, field.to))
            .collect();
        map
    }

    pub async fn init(&mut self) -> anyhow::Result<()> {
        // Use path that was passed in (via command line arguments)
        if self.path == None {
            self.path = mapping_path().await;
        }
        // Use a default path if it exists.
        // This checks for user system-dependent config path, e.g. on linux:
        // ~/.config/openaudiosearch/mapping.toml and /etc/openaudiosearch/mapping.toml
        if let Some(path) = &self.path {
            let contents = read_to_string(&path)
                .await
                .with_context(|| format!("File not found: {}", path.as_path().to_str().unwrap()))?;
            let mapping: HashMap<String, Mapping> = toml::from_str(&contents)?;
            self.mappings = mapping;
        // Use default mapping (included at compile time)
        } else {
            let mapping: HashMap<String, Mapping> = toml::from_str(DEFAULT_MAPPING)?;
            self.mappings = mapping;
        }
        Ok(())
    }
}

async fn mapping_path() -> Option<PathBuf> {
    let suffix = PathBuf::from(r"openaudiosearch/mapping.toml");
    if let Some(config_path) = config_dir() {
        let path = config_path.join(&suffix);
        if path_exists(&path).await {
            return Some(path);
        }
    }
    let path = PathBuf::from(r"/etc/openaudiosearch/mapping.toml").join(&suffix);
    if path_exists(&path).await {
        return Some(path);
    }

    None
}

async fn path_exists(path: &PathBuf) -> bool {
    let path = path.clone();
    let metadata = match metadata(&path).await {
        Ok(metadata) => metadata.is_file(),
        Err(e) => {
            log::debug!(
                "{} on path: {}",
                e,
                path.into_os_string().into_string().unwrap()
            );
            false
        }
    };
    metadata
}
