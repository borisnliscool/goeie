use crate::models::RedirectConfiguration;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::fs;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

type RedirectCache = (Instant, HashMap<String, RedirectConfiguration>);

lazy_static! {
    static ref CONFIG_CACHE: Arc<Mutex<Option<RedirectCache>>> = Arc::new(Mutex::new(None));
}

fn get_config_file() -> Result<String, String> {
    fs::read_to_string("config.toml").map_err(|e| e.to_string())
}

fn read_config_file() -> Result<HashMap<String, RedirectConfiguration>, String> {
    let config = get_config_file()?;
    toml::from_str(&config).map_err(|e| e.to_string())
}

pub fn get_host_config(host: String) -> Result<RedirectConfiguration, String> {
    let mut cache = CONFIG_CACHE.lock().unwrap();
    let now = Instant::now();

    // Check if cache is valid
    if let Some((timestamp, config)) = &*cache {
        if now.duration_since(*timestamp) < Duration::from_secs(300) {
            // 5 minutes
            return if let Some(value) = config.values().find(|value| value.hosts.contains(&host)) {
                // Clone the value to return an owned instance
                Ok(value.clone())
            } else {
                Err("host not found".to_string())
            };
        }
    }

    // Cache is invalid, read the config file
    let config = read_config_file()?;
    *cache = Some((now, config.clone()));

    // Find the host in the new config
    match config
        .into_values()
        .find(|value| value.hosts.contains(&host))
    {
        Some(value) => Ok(value),
        None => Err("host not found".to_string()),
    }
}
