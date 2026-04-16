use crate::db::Database;
use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use serde_json::json;

pub fn routes(db: Database) -> Router {
    Router::new()
        .route("/api/totals", get(totals))
        .route("/api/timeline", get(timeline))
        .route("/api/app-distribution", get(app_distribution))
        .route("/api/daily-avg", get(daily_avg))
        .with_state(db)
}

async fn totals(State(db): State<Database>) -> impl IntoResponse {
    match db.query_totals_all_time().await {
        Ok(totals) => Json(json!(totals)).into_response(),
        Err(e) => {
            tracing::error!("Error querying totals: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
                .into_response()
        }
    }
}

async fn timeline(State(db): State<Database>) -> impl IntoResponse {
    match db.query_24h_timeline().await {
        Ok(data) => Json(json!(data)).into_response(),
        Err(e) => {
            tracing::error!("Error querying timeline: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
                .into_response()
        }
    }
}

async fn app_distribution(State(db): State<Database>) -> impl IntoResponse {
    match db.query_app_distribution(30).await {
        Ok(data) => Json(json!(data)).into_response(),
        Err(e) => {
            tracing::error!("Error querying app distribution: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
                .into_response()
        }
    }
}

async fn daily_avg(State(db): State<Database>) -> impl IntoResponse {
    match db.query_daily_avg_stats().await {
        Ok(data) => Json(json!(data)).into_response(),
        Err(e) => {
            tracing::error!("Error querying daily averages: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
                .into_response()
        }
    }
}
