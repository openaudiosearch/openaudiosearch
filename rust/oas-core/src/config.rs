use dirs::config_dir;
use serde::de::DeserializeOwned;
use std::path::Path;

pub const CONFIG_PREFIX: &str = "openaudiosearch";

pub async fn load_config<C>(name: impl ToString, default: Option<&[u8]>) -> anyhow::Result<C>
where
    C: DeserializeOwned,
{
    let mut locations = vec![];
    let name = name.to_string();
    let location = Path::new(CONFIG_PREFIX).join(Path::new(&name));
    if let Some(dir) = config_dir() {
        let dir = dir.join(location.clone());
        locations.push(dir);
    }
    locations.push(Path::new("/etc").join(location));
    eprintln!("locs {:#?}", locations);

    let buf = loop {
        if let Some(location) = locations.pop() {
            if let Ok(buf) = tokio::fs::read(&location).await {
                log::debug!(
                    "loading config `{}` from file `{}`",
                    name,
                    location.display()
                );
                break Some(buf);
            }
        } else {
            break None;
        }
    };
    let buf = if buf.is_some() {
        buf
    } else {
        log::debug!("loading config `{}` from default", name);
        default.map(|buf| buf.to_vec())
    };
    let buf =
        buf.ok_or_else(|| anyhow::anyhow!("No config file found and no default provided."))?;
    let config: C = toml::from_slice(&buf[..])?;
    Ok(config)
}
