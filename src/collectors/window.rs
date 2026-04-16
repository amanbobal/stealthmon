use crate::db::Database;
use chrono::Utc;
use tokio_util::sync::CancellationToken;

/// Privacy-sensitive app names — store as "private" with no window title.
const PRIVATE_APPS: &[&str] = &[
    "keepass",
    "1password",
    "bitwarden",
    "lastpass",
    "banking",
    "wallet",
];

/// Categorise an app by its process name and optional process path.
pub fn categorise(app: &str, process_path: Option<&str>) -> &'static str {
    let a = app.to_lowercase();

    if ["code", "antigravity", "zed", "cursor", "neovide", "nvim", "vim", "clion",
        "intellij", "rider", "fleet", "sublime_text", "notepad++"]
        .iter()
        .any(|k| a.contains(k))
    {
        return "coding";
    }
    if ["steam", "epicgameslauncher", "minecraft", "javaw", "valorant",
        "league of legends", "csgo", "cs2", "overwatch", "fortnite",
        "roblox", "genshin", "origin", "battle.net"]
        .iter()
        .any(|k| a.contains(k))
    {
        return "gaming";
    }
    if ["chrome", "firefox", "msedge", "opera", "brave", "vivaldi",
        "safari", "arc"]
        .iter()
        .any(|k| a.contains(k))
    {
        return "browser";
    }
    if ["discord", "slack", "teams", "telegram", "whatsapp", "signal",
        "zoom", "skype"]
        .iter()
        .any(|k| a.contains(k))
    {
        return "communication";
    }
    if ["mpv", "vlc", "mpc-hc", "netflix", "spotify", "youtube",
        "crunchyroll"]
        .iter()
        .any(|k| a.contains(k))
    {
        return "media";
    }
    if ["blender", "figma", "photoshop", "illustrator", "davinci",
        "premiere", "after effects", "krita", "gimp"]
        .iter()
        .any(|k| a.contains(k))
    {
        return "creative";
    }
    if ["word", "excel", "powerpoint", "notion", "obsidian", "onenote",
        "libreoffice"]
        .iter()
        .any(|k| a.contains(k))
    {
        return "productivity";
    }

    // Auto-categorize apps running from E: drive as gaming
    if let Some(path) = process_path {
        let p = path.to_lowercase();
        if p.starts_with("e:\\") || p.starts_with("e:/") {
            return "gaming";
        }
    }

    "other"
}

/// Check if an app should be treated as private.
fn is_private(app: &str) -> bool {
    let a = app.to_lowercase();
    PRIVATE_APPS.iter().any(|k| a.contains(k))
}

/// Poll the active window every 5 seconds and record snapshots + app time.
pub async fn start_window_collector(db: Database, cancel: CancellationToken) {
    tracing::info!("Starting active window collector (5s poll)");
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));

    loop {
        tokio::select! {
            _ = cancel.cancelled() => {
                tracing::info!("Window collector shutting down");
                break;
            }
            _ = interval.tick() => {
                // active-win-pos-rs is blocking, run in spawn_blocking
                let result = tokio::task::spawn_blocking(|| {
                    active_win_pos_rs::get_active_window()
                }).await;

                match result {
                    Ok(Ok(window)) => {
                        let raw_app = window.app_name;
                        let raw_title = window.title;
                        let process_path = window.process_path
                            .to_string_lossy()
                            .to_string();

                        let (app_name, window_title) = if is_private(&raw_app) {
                            ("private".to_string(), None)
                        } else {
                            (raw_app, Some(raw_title))
                        };

                        let category = categorise(&app_name, Some(&process_path));
                        let date_bucket = Utc::now().format("%Y-%m-%d").to_string();

                        // Insert snapshot
                        if let Err(e) = db
                            .insert_window_snapshot(
                                &app_name,
                                window_title.as_deref(),
                                category,
                            )
                            .await
                        {
                            tracing::error!("Failed to insert window snapshot: {}", e);
                        }

                        // Upsert daily app time (+5 seconds)
                        if let Err(e) = db
                            .upsert_daily_app_time(&date_bucket, &app_name, category, 5)
                            .await
                        {
                            tracing::error!("Failed to upsert daily app time: {}", e);
                        }
                    }
                    Ok(Err(e)) => {
                        tracing::debug!("Could not get active window: {:?}", e);
                    }
                    Err(e) => {
                        tracing::error!("spawn_blocking panicked: {}", e);
                    }
                }
            }
        }
    }
}
