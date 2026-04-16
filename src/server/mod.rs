pub mod api;
pub mod dashboard;

use crate::db::Database;
use axum::Router;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;
use tower_http::cors::CorsLayer;

/// Start the axum HTTP server on 127.0.0.1:9521.
pub async fn start_server(db: Database, cancel: CancellationToken) {
    let app = Router::new()
        .merge(dashboard::routes())
        .merge(api::routes(db))
        .layer(CorsLayer::permissive());

    let addr = SocketAddr::from(([127, 0, 0, 1], 9521));
    tracing::info!("Dashboard server listening on http://{}", addr);

    let listener = match TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => {
            tracing::error!("Failed to bind HTTP server to {}: {}", addr, e);
            return;
        }
    };

    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            cancel.cancelled().await;
            tracing::info!("HTTP server shutting down");
        })
        .await
        .unwrap_or_else(|e| {
            tracing::error!("HTTP server error: {}", e);
        });
}
