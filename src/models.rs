use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub enum RedirectType {
    Temporary,
    Permanent,
}

#[derive(Debug, Deserialize, Clone)]
pub enum PathOption {
    Keep,
    Remove,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RedirectConfiguration {
    pub hosts: Vec<String>,
    pub target: String,
    pub redirect_type: Option<RedirectType>,
    pub path: Option<PathOption>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PortConfiguration {
    pub http: u16,
    pub https: u16,
}

impl Default for PortConfiguration {
    fn default() -> Self {
        PortConfiguration {
            http: 80,
            https: 443,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub ports: Option<PortConfiguration>,
    pub redirect: Vec<RedirectConfiguration>,
}
