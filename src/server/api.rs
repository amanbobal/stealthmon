use crate::db::{Database, WebHistoryVisit};
use crate::updater::UpdateManager;
use axum::{
    extract::{DefaultBodyLimit, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Extension,
    Json, Router,
};
use serde::Deserialize;
use serde_json::json;
use winreg::enums::*;
use winreg::RegKey;

#[derive(Debug, Deserialize)]
pub struct RangeParams {
    pub range: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum WebHistoryPayload {
    Wrapped { visits: Vec<WebHistoryVisit> },
    Visits(Vec<WebHistoryVisit>),
}

pub fn routes(db: Database, updater: UpdateManager) -> Router {
    Router::new()
        .route("/api/username", get(get_username))
        .route("/api/totals", get(totals))
        .route("/api/timeline", get(timeline))
        .route("/api/app-distribution", get(app_distribution))
        .route("/api/daily-avg", get(daily_avg))
        .route("/api/characters", get(characters))
        .route("/api/web-history", post(ingest_web_history))
        .route("/api/web-history/most-visited", get(most_visited_website))
        .route("/api/web-history/status", get(web_history_status))
        .route("/api/update/status", get(update_status))
        .route("/api/update/check", post(update_check))
        .route("/api/update/install", post(update_install))
        .route("/api/startup", get(get_startup).post(set_startup))
        .layer(Extension(updater))
        .layer(DefaultBodyLimit::max(50 * 1024 * 1024))
        .with_state(db)
}

async fn get_username() -> impl IntoResponse {
    let username = std::env::var("USERNAME").unwrap_or_else(|_| "Pardon".to_string());
    Json(json!({ "username": username })).into_response()
}

fn parse_range(params: &RangeParams) -> &str {
    match params.range.as_deref() {
        Some("week") | Some("7d") => "7d",
        Some("month") | Some("30d") => "30d",
        Some("year") | Some("365d") => "365d",
        _ => "24h",
    }
}

fn range_to_hours(range: &str) -> u32 {
    match range {
        "7d" => 24 * 7,
        "30d" => 24 * 30,
        "365d" => 24 * 365,
        _ => 24,
    }
}

fn range_to_days(range: &str) -> u32 {
    match range {
        "7d" => 7,
        "30d" => 30,
        "365d" => 365,
        _ => 1,
    }
}

async fn totals(
    State(db): State<Database>,
    Query(params): Query<RangeParams>,
) -> impl IntoResponse {
    let range = parse_range(&params);
    let hours_back = range_to_hours(range);
    match db.query_totals_range(hours_back).await {
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

async fn timeline(
    State(db): State<Database>,
    Query(params): Query<RangeParams>,
) -> impl IntoResponse {
    let range = parse_range(&params);
    let hours_back = range_to_hours(range);
    match db.query_timeline_range(hours_back).await {
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

async fn app_distribution(
    State(db): State<Database>,
    Query(params): Query<RangeParams>,
) -> impl IntoResponse {
    let range = parse_range(&params);
    let days_back = range_to_days(range);
    match db.query_app_distribution(days_back).await {
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

async fn daily_avg(
    State(db): State<Database>,
    Query(params): Query<RangeParams>,
) -> impl IntoResponse {
    let range = parse_range(&params);
    let days_back = range_to_days(range);
    match db.query_daily_stats(days_back).await {
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

async fn characters(
    State(db): State<Database>,
    Query(params): Query<RangeParams>,
) -> impl IntoResponse {
    let range = parse_range(&params);
    let hours_back = range_to_hours(range);
    match db.query_character_stats(hours_back).await {
        Ok(data) => Json(json!(data)).into_response(),
        Err(e) => {
            tracing::error!("Error querying characters: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
                .into_response()
        }
    }
}

async fn ingest_web_history(
    State(db): State<Database>,
    Json(payload): Json<WebHistoryPayload>,
) -> impl IntoResponse {
    let visits = match payload {
        WebHistoryPayload::Wrapped { visits } => visits,
        WebHistoryPayload::Visits(visits) => visits,
    };

    match db.upsert_web_history(visits).await {
        Ok(imported) => Json(json!({ "success": true, "imported": imported })).into_response(),
        Err(e) => {
            tracing::error!("Error ingesting web history: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
                .into_response()
        }
    }
}

async fn most_visited_website(
    State(db): State<Database>,
    Query(params): Query<RangeParams>,
) -> impl IntoResponse {
    let range = parse_range(&params);
    let hours_back = range_to_hours(range);
    match db.query_most_visited_website(hours_back).await {
        Ok(data) => Json(json!(data)).into_response(),
        Err(e) => {
            tracing::error!("Error querying most visited website: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
                .into_response()
        }
    }
}

async fn web_history_status(State(db): State<Database>) -> impl IntoResponse {
    match db.query_web_history_status().await {
        Ok(data) => Json(json!(data)).into_response(),
        Err(e) => {
            tracing::error!("Error querying web history status: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
                .into_response()
        }
    }
}

async fn update_status(Extension(updater): Extension<UpdateManager>) -> impl IntoResponse {
    Json(json!(updater.status().await)).into_response()
}

async fn update_check(Extension(updater): Extension<UpdateManager>) -> impl IntoResponse {
    Json(json!(updater.check_for_updates().await)).into_response()
}

async fn update_install(Extension(updater): Extension<UpdateManager>) -> impl IntoResponse {
    match updater.install_update().await {
        Ok(status) => Json(json!(status)).into_response(),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": error })),
        )
            .into_response(),
    }
}

async fn get_startup() -> impl IntoResponse {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let run = hkcu.open_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Run");
    let enabled = if let Ok(run) = run {
        let val: Result<String, _> = run.get_value("StealthMon");
        val.is_ok()
    } else {
        false
    };
    Json(json!({ "enabled": enabled })).into_response()
}

#[derive(Deserialize)]
struct StartupPayload {
    enabled: bool,
}

async fn set_startup(Json(payload): Json<StartupPayload>) -> impl IntoResponse {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let run = hkcu.open_subkey_with_flags(
        "Software\\Microsoft\\Windows\\CurrentVersion\\Run",
        KEY_SET_VALUE,
    );
    if let Ok(run) = run {
        if payload.enabled {
            if let Ok(exe_path) = std::env::current_exe() {
                let _ = run.set_value("StealthMon", &exe_path.to_string_lossy().to_string());
            }
        } else {
            let _ = run.delete_value("StealthMon");
        }
        Json(json!({ "success": true })).into_response()
    } else {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Failed to open registry"})),
        )
            .into_response()
    }
}
