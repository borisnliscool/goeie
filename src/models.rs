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
