use crate::models::{Config, RedirectConfiguration};
use lazy_static::lazy_static;
use std::{env, fs};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

type ConfigCache = (Instant, Config);

lazy_static! {
    static ref CONFIG_CACHE: Arc<Mutex<Option<ConfigCache>>> = Arc::new(Mutex::new(None));
}

fn get_config_file() -> Result<String, String> {
    fs::read_to_string(
        Path::join(
            &env::current_dir().unwrap(),
            Path::new("config.toml")
        )
    ).map_err(|e| e.to_string())
}

fn read_config_file() -> Result<Config, String> {
    let config = get_config_file()?;
    toml::from_str(&config).map_err(|e| e.to_string())
}

pub fn get_config() -> Result<Config, String> {
    let mut cache = CONFIG_CACHE.lock().unwrap();
    let now = Instant::now();

    // Check if cache is valid
    if let Some((timestamp, config)) = &*cache {
        if now.duration_since(*timestamp) < Duration::from_secs(300) {
            return Ok(config.clone());
        }
    }

    // Cache is invalid, read the config file
    let config = read_config_file()?;
    *cache = Some((now, config.clone()));
    tracing::info!("Updated config cache");

    Ok(config)
}

pub fn get_host_config(host: String) -> Result<RedirectConfiguration, String> {
    let config = get_config()?;

    match config
        .redirect
        .into_iter()
        .find(|value| value.hosts.contains(&host))
    {
        Some(value) => Ok(value),
        None => Err("host not found".to_string()),
    }
}
