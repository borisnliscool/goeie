use crate::models::{RedirectConfiguration};
use std::collections::HashMap;
use std::fs;

fn get_config_file() -> Result<String, String> {
    fs::read_to_string("config.toml").map_err(|e| e.to_string())
}

pub fn get_host_config(host: String) -> Result<RedirectConfiguration, String> {
    let config = get_config_file()?;
    let config: HashMap<String, RedirectConfiguration> =
        toml::from_str(&config).map_err(|e| e.to_string())?;

    match config.into_values().find(|value| value.hosts.contains(&host)) {
        Some(value) => Ok(value),
        None => Err("host not found".to_string()),
    }
}
