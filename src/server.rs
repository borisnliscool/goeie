use crate::certs::get_domain_cert;
use crate::config;
use crate::models::{Config, RedirectType};
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing::get;
use axum::Router;
use axum_server::tls_rustls::RustlsConfig;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

async fn handle_request(headers: HeaderMap) -> impl IntoResponse {
    let host = headers.get("host").and_then(|h| h.to_str().ok());
    if host.is_none() {
        return Response::builder()
            .status(400)
            .body("Missing host header".into())
            .unwrap();
    }

    let host_config = config::get_host_config(host.unwrap().to_string());
    if host_config.is_err() {
        return Response::builder()
            .status(400)
            .body("No configuration found".into())
            .unwrap();
    }

    let config = host_config.unwrap();
    tracing::info!("Redirecting {} to {}", host.unwrap(), config.target);

    let mut headers = HeaderMap::new();
    headers.insert("X-Powered-By", "Goeie".parse().unwrap());

    let redirect = match config.redirect_type.unwrap_or(RedirectType::Temporary) {
        RedirectType::Temporary => Redirect::temporary(&config.target),
        RedirectType::Permanent => Redirect::permanent(&config.target),
    };

    (headers, redirect).into_response()
}

pub async fn start_tls_server(config: Config) {
    let addr = SocketAddr::from(([0, 0, 0, 0], config.clone().ports.unwrap_or_default().https));
    tracing::info!("TLS server listening on {}", addr);

    let (cert, key) = get_domain_cert(config.clone(), "example.com").unwrap();
    let config = RustlsConfig::from_pem(cert.into_bytes(), key.into_bytes())
        .await
        .unwrap();

    let server = Router::new()
        .fallback(get(handle_request))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive());

    axum_server::bind_rustls(addr, config)
        .serve(server.into_make_service())
        .await
        .expect("Failed to serve");
}

pub async fn start_http_server(config: Config) {
    let addr = SocketAddr::from(([0, 0, 0, 0], config.ports.unwrap_or_default().http));

    let listener = TcpListener::bind(addr).await.expect("Failed to bind");
    tracing::info!("HTTP server listening on {}", addr);

    let server = Router::new()
        .fallback(get(handle_request))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive());

    axum::serve(listener, server)
        .await
        .expect("Failed to serve");
}
