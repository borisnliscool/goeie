mod config;
mod models;

use crate::models::RedirectType;
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing::get;
use axum::Router;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

async fn handle_request(headers: HeaderMap) -> Response {
    let host = headers.get("host").and_then(|h| h.to_str().ok());
    if host.is_none() {
        return Response::builder()
            .status(400)
            .body("Missing host header".into())
            .unwrap();
    }

    let config = config::get_host_config(host.unwrap().to_string());
    if config.is_err() {
        return Response::builder()
            .status(400)
            .body("No configuration found".into())
            .unwrap();
    }

    let config = config.unwrap();
    tracing::info!("Redirecting {} to {}", host.unwrap(), config.target);

    match config.redirect_type.unwrap_or(RedirectType::Temporary) {
        RedirectType::Temporary => Redirect::temporary(&config.target).into_response(),
        RedirectType::Permanent => Redirect::permanent(&config.target).into_response(),
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 3000));
    let listener = TcpListener::bind(addr).await.expect("Failed to bind");
    tracing::info!("Webserver listening on {}", addr);

    let server = Router::new()
        .fallback(get(handle_request))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive());

    axum::serve(listener, server)
        .await
        .expect("Failed to serve");
}
