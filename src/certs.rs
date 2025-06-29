use crate::models::{Config, TLSConfiguration};
use acme_micro::{create_p384_key, Account, Directory, DirectoryUrl};
use axum::routing::get;
use axum::Router;
use std::collections::HashSet;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use std::{env, fs};
use tokio::net::TcpListener;
use tokio::sync::{oneshot, Mutex};

fn get_acme_account(tls_config: TLSConfiguration) -> Result<Account, String> {
    // todo: configurable
    let url = DirectoryUrl::LetsEncryptStaging;
    let dir = Directory::from_url(url).map_err(|e| e.to_string())?;

    let contact = vec![format!("mailto:{}", tls_config.email)];

    let credentials_path = Path::join(
        &env::current_dir().map_err(|e| e.to_string())?,
        Path::join(tls_config.certs_path.as_ref(), "private.pem"),
    );

    Ok(if fs::exists(credentials_path.clone()).unwrap_or(false) {
        let key = fs::read_to_string(credentials_path).map_err(|e| e.to_string())?;
        dir.load_account(&key, contact).map_err(|e| e.to_string())?
    } else {
        tracing::info!("Creating credentials file");

        let acc = dir
            .register_account(contact.clone())
            .map_err(|e| e.to_string())?;

        let key = acc.acme_private_key_pem().map_err(|e| e.to_string())?;
        fs::write(credentials_path, key).map_err(|e| e.to_string())?;
        acc
    })
}

async fn request_cert(config: Config, domain: &str) -> Result<String, String> {
    tracing::info!("Requesting certificate for {}", domain);

    let acc = get_acme_account(config.tls.clone().unwrap())?;
    let ord_new = acc.new_order(domain, &[]).map_err(|e| e.to_string())?;

    let ord_csr = loop {
        if let Some(ord_csr) = ord_new.confirm_validations() {
            break ord_csr;
        }

        let auths = ord_new.authorizations().map_err(|e| e.to_string())?;
        let challenge = auths[0].http_challenge().unwrap();

        let path = format!("/.well-known/acme-challenge/{}", challenge.http_token());
        let _ = run_acme_server(config.clone(), path, challenge.http_proof().unwrap()).await;
    };

    let pkey_pri = create_p384_key().map_err(|e| e.to_string())?;
    let ord_cert = ord_csr
        .finalize_pkey(pkey_pri, Duration::from_millis(5000))
        .map_err(|e| e.to_string())?;
    let cert = ord_cert.download_cert().map_err(|e| e.to_string())?;

    let tls_config = config.tls.clone().unwrap();
    let cert_path = Path::new(&tls_config.certs_path).join(format!("{domain}.crt"));
    let key_path = Path::new(&tls_config.certs_path).join(format!("{domain}.key"));

    fs::write(cert_path, cert.certificate()).map_err(|e| e.to_string())?;
    fs::write(key_path, cert.private_key()).map_err(|e| e.to_string())?;

    Ok(cert.certificate().parse().unwrap())
}

pub async fn register_domain_certs(domains: Vec<String>, config: Config) {
    let domains: HashSet<String> = HashSet::from_iter(domains.iter().cloned());

    for domain in domains {
        request_cert(config.clone(), &domain).await.unwrap();
    }
}

pub fn get_domain_cert(config: Config, domain: &str) -> Result<(String, String), String> {
    let tls_config = config.tls.clone().unwrap();
    let cert_path = Path::new(&tls_config.certs_path).join(format!("{domain}.crt"));
    let key_path = Path::new(&tls_config.certs_path).join(format!("{domain}.key"));

    if !fs::exists(cert_path.clone()).unwrap_or(false) {
        // TODO: maybe request a cert here?
        return Err("Certificate not found".to_string());
    }

    if !fs::exists(key_path.clone()).unwrap_or(false) {
        // TODO: maybe request a cert here?
        return Err("Key not found".to_string());
    }

    let cert = fs::read_to_string(cert_path).map_err(|e| e.to_string())?;
    let key = fs::read_to_string(key_path).map_err(|e| e.to_string())?;

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
                        let _ = sender.send("Request received!");
                    }

                    content
                }
            }
        }),
    );

    tokio::spawn(async move {
        let addr = SocketAddr::from(([0, 0, 0, 0], config.ports.unwrap_or_default().http));
        let listener = TcpListener::bind(addr).await.expect("Failed to bind");

        if let Err(e) = axum::serve(listener, server).await {
            tracing::error!("Server error: {}", e);
        }
    });

    match rx.await {
        Ok(message) => {
            tracing::info!("Message from server: {}", message);
            Some(message.to_string())
        }
        Err(e) => {
            tracing::error!("Error receiving message: {}", e);
            None
        }
    }
}
