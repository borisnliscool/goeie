mod dns;
mod models;

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

    let config = dns::get_redirect_config(host.unwrap()).await;
    if config.is_err() {
        return Response::builder()
            .status(400)
            .body("No goeie configuration found".into())
            .unwrap();
    }

    let config = config.unwrap();
    tracing::debug!("Goeie configuration: {:?}", config);
    Redirect::temporary(&config.redirect_target_url).into_response()
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
