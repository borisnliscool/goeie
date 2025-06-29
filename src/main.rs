mod certs;
mod config;
mod models;
mod server;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        // todo: configurable
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let config = config::get_config();
    if config.is_err() {
        tracing::error!("{}", config.unwrap_err());
        std::process::exit(1);
    }

    let config = config.unwrap();

    if config.clone().tls.is_some() {
        certs::register_domain_certs(
            config
                .clone()
                .redirect
                .iter()
                .map(|r| r.hosts.iter().map(|h| h.to_string()).collect())
                .collect(),
            config.clone(),
        ).await;
        tokio::spawn(server::start_tls_server(config.clone()));
    }

    server::start_http_server(config).await;
}
