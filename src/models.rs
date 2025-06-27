use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub enum RedirectType {
    Temporary,
    Permanent,
}

#[derive(Debug, Deserialize)]
pub struct RedirectConfiguration {
    pub hosts: Vec<String>,
    pub target: String,
    pub redirect_type: Option<RedirectType>,
}
