use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct RedirectConfiguration {
    pub target: String,
}
