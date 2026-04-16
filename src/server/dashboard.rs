use axum::{
    http::{header, StatusCode},
    response::IntoResponse,
    routing::get,
    Router,
};
use rust_embed::Embed;

#[derive(Embed)]
#[folder = "assets/"]
struct Assets;

pub fn routes() -> Router {
    Router::new()
        .route("/", get(serve_dashboard))
        .route("/assets/*path", get(serve_asset))
}

async fn serve_dashboard() -> impl IntoResponse {
    match Assets::get("dashboard.html") {
        Some(content) => {
            let body = content.data.to_vec();
            (
                StatusCode::OK,
                [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
                body,
            )
                .into_response()
        }
        None => (StatusCode::NOT_FOUND, "Dashboard not found").into_response(),
    }
}

async fn serve_asset(axum::extract::Path(path): axum::extract::Path<String>) -> impl IntoResponse {
    match Assets::get(&path) {
        Some(content) => {
            let mime = if path.ends_with(".js") {
                "application/javascript"
            } else if path.ends_with(".css") {
                "text/css"
            } else if path.ends_with(".ico") {
                "image/x-icon"
            } else if path.ends_with(".png") {
                "image/png"
            } else if path.ends_with(".svg") {
                "image/svg+xml"
            } else {
                "application/octet-stream"
            };

            let body = content.data.to_vec();
            (StatusCode::OK, [(header::CONTENT_TYPE, mime)], body).into_response()
        }
        None => (StatusCode::NOT_FOUND, "Asset not found").into_response(),
    }
}
