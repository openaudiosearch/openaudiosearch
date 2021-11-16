use std::time::Duration;

use crate::{Record, TypedValue};

pub const RETRY_INTERVAL: Duration = Duration::from_secs(1);
pub const MAX_RETRIES: usize = 120;

pub fn debug_print_records<T>(records: &[Record<T>])
where
    T: TypedValue,
{
    for record in records {
        debug_print_record(record)
    }
}
pub fn debug_print_record<T>(record: &Record<T>)
where
    T: TypedValue,
{
    eprintln!(
        r#"<Record {}_{} [{}]>"#,
        record.id(),
        record.typ(),
        record.value.label().unwrap_or_default()
    );
}

pub struct RetryOpts {
    pub max_retries: usize,
    pub name: Option<String>,
    pub interval: Duration,
}
impl Default for RetryOpts {
    fn default() -> Self {
        Self {
            max_retries: MAX_RETRIES,
            interval: RETRY_INTERVAL,
            name: None,
        }
    }
}
impl RetryOpts {
    pub fn with_name(name: String) -> Self {
        Self {
            name: Some(name),
            ..Default::default()
        }
    }
}

/// Repeat a HTTP request until it returns a successfull status.
pub async fn wait_for_ready(
    client: &reqwest::Client,
    opts: RetryOpts,
    req_builder: impl Fn() -> Result<reqwest::Request, reqwest::Error>,
) -> Result<(), std::io::Error> {
    let mut interval = tokio::time::interval(opts.interval);
    let name = opts.name.unwrap_or_default();
    for _i in 0..opts.max_retries {
        let req = req_builder()
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, format!("{}", err)))?;
        let url = req.url().to_string();
        match client.execute(req).await {
            Ok(res) => {
                if res.status().is_success() {
                    return Ok(());
                } else {
                    log::warn!(
                        "Failed to connect to {} at {}: {}",
                        name,
                        url,
                        res.status().canonical_reason().unwrap()
                    );
                }
            }
            Err(err) => {
                log::warn!("Failed to connect to {} at {}: {}", name, url, err);
            }
        }
        interval.tick().await;
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::TimedOut,
        "Cannot reach service {}",
    ))
}
