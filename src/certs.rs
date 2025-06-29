use crate::models::{Config, TLSConfiguration};
use acme_micro::{create_p384_key, Account, Directory, DirectoryUrl};
use axum::{routing::get, Router};
use std::{
    collections::HashSet,
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};
use tokio::{
    net::TcpListener,
    sync::{oneshot, Mutex},
};
use tracing::{error, info};

fn get_acme_account(tls_config: &TLSConfiguration) -> Result<Account, String> {
    let url = DirectoryUrl::LetsEncrypt;
    let dir = Directory::from_url(url).map_err(|e| e.to_string())?;
    let contact = vec![format!("mailto:{}", tls_config.email)];

    let credentials_path = get_credentials_path(tls_config)?;

    if credentials_path.exists() {
        let key = std::fs::read_to_string(&credentials_path).map_err(|e| e.to_string())?;
        dir.load_account(&key, contact).map_err(|e| e.to_string())
    } else {
        info!("Creating credentials file");
        let acc = dir.register_account(contact).map_err(|e| e.to_string())?;
        let key = acc.acme_private_key_pem().map_err(|e| e.to_string())?;
        std::fs::write(&credentials_path, key).map_err(|e| e.to_string())?;
        Ok(acc)
    }
}

fn get_credentials_path(tls_config: &TLSConfiguration) -> Result<PathBuf, String> {
    let current_dir = std::env::current_dir().map_err(|e| e.to_string())?;
    Ok(current_dir.join(&tls_config.certs_path).join("private.pem"))
}

async fn request_cert(config: Config, domain: &str) -> Result<String, String> {
    info!("Requesting certificate for {}", domain);
    let tls_config = config.clone().tls.ok_or("TLS configuration not found")?;
    let acc = get_acme_account(&tls_config)?;

    let ord_new = acc.new_order(domain, &[]).map_err(|e| e.to_string())?;
    let ord_csr = loop {
        if let Some(ord_csr) = ord_new.confirm_validations() {
            break ord_csr;
        }

        let auths = ord_new.authorizations().map_err(|e| e.to_string())?;
        let challenge = match auths[0].http_challenge() {
            None => Err("No HTTP challenge found".to_string()),
            Some(token) => Ok(token),
        }?;

        let path = format!("/.well-known/acme-challenge/{}", challenge.http_token());

        if run_acme_server(
            config.clone(),
            path,
            challenge.http_proof().map_err(|e| e.to_string())?,
        )
        .await
        .is_none()
        {
            return Err("ACME server failed to respond".to_string());
        }
    };

    let pkey_pri = create_p384_key().map_err(|e| e.to_string())?;
    let ord_cert = ord_csr
        .finalize_pkey(pkey_pri, Duration::from_millis(5000))
        .map_err(|e| e.to_string())?;

    let cert = ord_cert.download_cert().map_err(|e| e.to_string())?;

    let cert_path = Path::new(&tls_config.certs_path).join(format!("{domain}.crt"));
    let key_path = Path::new(&tls_config.certs_path).join(format!("{domain}.key"));

    std::fs::write(&cert_path, cert.certificate()).map_err(|e| e.to_string())?;
    std::fs::write(&key_path, cert.private_key()).map_err(|e| e.to_string())?;

    Ok(cert.certificate().to_string())
}

pub async fn register_domain_certs(domains: Vec<String>, config: Config) {
    let domains: HashSet<String> = HashSet::from_iter(domains.into_iter());
    for domain in domains {
        if let Err(e) = request_cert(config.clone(), &domain).await {
            error!("Failed to request certificate for {}: {}", domain, e);
        }
    }
}

pub fn get_domain_cert(config: Config, domain: &str) -> Result<(String, String), String> {
    let tls_config = config.tls.ok_or("TLS configuration not found")?;
    let cert_path = Path::new(&tls_config.certs_path).join(format!("{domain}.crt"));
    let key_path = Path::new(&tls_config.certs_path).join(format!("{domain}.key"));

    if !cert_path.exists() || !key_path.exists() {
        return Err(format!("Certificate or key not found for domain {domain}",));
    }

    let cert = std::fs::read_to_string(&cert_path).map_err(|e| e.to_string())?;
    let key = std::fs::read_to_string(&key_path).map_err(|e| e.to_string())?;

    Ok((cert, key))
}

async fn run_acme_server(config: Config, path: String, content: String) -> Option<String> {
    let (tx, rx) = oneshot::channel();
    let tx = Arc::new(Mutex::new(Some(tx)));

    let server = Router::new().route(
        &path,
        get({
            let tx = Arc::clone(&tx);
            move || {
                let tx = Arc::clone(&tx);
                async move {
                    if let Some(sender) = tx.lock().await.take() {
                        let _ = sender.send("Request received!".to_string());
                    }
                    content.clone()
                }
            }
        }),
    );

    let addr = SocketAddr::from(([0, 0, 0, 0], config.ports.unwrap_or_default().http));
    let tx_clone = Arc::clone(&tx);

    tokio::spawn(async move {
        let listener = match TcpListener::bind(addr).await {
            Ok(l) => l,
            Err(e) => {
                error!("Failed to bind to address: {}", e);
                if let Some(sender) = tx_clone.lock().await.take() {
                    let _ = sender.send(format!("Bind error: {e}"));
                }
                return;
            }
        };

        if let Err(e) = axum::serve(listener, server).await {
            error!("Server error: {}", e);
        }
    });

    match rx.await {
        Ok(message) => {
            info!("Message from server: {}", message);
            Some(message)
        }
        Err(e) => {
            error!("Error receiving message: {}", e);
            None
        }
    }
}
