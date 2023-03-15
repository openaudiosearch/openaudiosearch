use serde::Deserialize;
use std::collections::HashMap;

use super::argfuns::{ArgFunContext, ArgFunctions};

const DEFAULT_CONFIG: &'static [u8] = include_bytes!("../../../../config/jobs.toml");

#[derive(Deserialize, Debug, Clone, Default)]
pub struct JobConfig {
    pub on_complete: HashMap<String, Vec<StartConfig>>,
}

impl JobConfig {
    pub async fn load() -> anyhow::Result<Self> {
        crate::config::load_config("jobs.toml", Some(DEFAULT_CONFIG)).await
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct StartConfig {
    pub job: String,
    pub args: HashMap<String, serde_json::Value>,
}

impl StartConfig {
    pub async fn template_to_args(
        &self,
        args: &serde_json::Value,
        argfuns: &ArgFunctions,
        argfun_ctx: ArgFunContext,
    ) -> anyhow::Result<serde_json::Value> {
        let mut template = self.args.clone();
        let input = serde_json::json!({ "args": args });
        for (_key, value) in template.iter_mut() {
            match value {
                serde_json::Value::String(string) => {
                    if string.starts_with("{{") && string.ends_with("}}") {
                        let pattern = &string[2..(string.len() - 2)];
                        let parts: Vec<_> = pattern.split("|").collect();
                        let getter = parts.get(0).unwrap();
                        let argfun = parts.get(1);

                        let replacement = extract_replacement(&input, &getter);

                        if let Some(replacement) = replacement {
                            // Run argfuns if present.
                            let replacement = if let Some(argfun) = argfun {
                                eprintln!("run!");
                                argfuns
                                    .apply(argfun_ctx.clone(), argfun, replacement)
                                    .await?
                            } else {
                                replacement
                            };

                            // Replace the value.
                            *value = replacement
                        }
                    }
                }
                _ => {}
            }
        }
        Ok(serde_json::to_value(template)?)
    }
}

fn extract_replacement(input: &serde_json::Value, pattern: &str) -> Option<serde_json::Value> {
    let pointer = pattern.replace(".", "/");
    let pointer = format!("/{}", pointer);
    let res = input.pointer(&pointer);
    let res = res.map(|r| r.clone());
    res
}
