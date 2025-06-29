use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub enum RedirectType {
    Temporary,
    Permanent,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RedirectConfiguration {
    pub hosts: Vec<String>,
    pub target: String,
    pub redirect_type: Option<RedirectType>,
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
pub struct TLSConfiguration {
    pub email: String,
    pub certs_path: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub redirect: Vec<RedirectConfiguration>,
    pub ports: Option<PortConfiguration>,
    pub tls: Option<TLSConfiguration>,
}
