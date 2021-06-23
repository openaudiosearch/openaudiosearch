use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Engine {
    Vosk,
    Pytorch,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AsrArgs {
    pub media_file: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AsrOpts {
    pub engine: Engine,
    pub language: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct AsrResult {
    pub text: String,
    pub parts: Vec<serde_json::Value>,
}
