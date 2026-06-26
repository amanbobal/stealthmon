use chrono::Utc;
use rusqlite::{params, Connection as SqliteConnection};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};
use tokio_rusqlite::Connection;

/// Async database wrapper for all activity monitor operations.
#[derive(Clone)]
pub struct Database {
    conn: Arc<Connection>,
}

#[derive(Debug, Serialize, Clone)]
pub struct CharacterStat {
    pub character: String,
    pub count: i64,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Clone)]
pub struct HourlyStat {
    pub hour_bucket: String,
    pub keypresses: i64,
    pub left_clicks: i64,
    pub right_clicks: i64,
    pub middle_clicks: i64,
    pub mouse_feet: f64,
    pub controller_buttons: i64,
}

#[derive(Debug, Serialize, Clone)]
pub struct DailyStat {
    pub day: String,
    pub left_clicks: i64,
    pub right_clicks: i64,
    pub middle_clicks: i64,
    pub keypresses: i64,
    pub mouse_feet: f64,
    pub controller_buttons: i64,
}

#[derive(Debug, Serialize, Clone)]
pub struct AppShare {
    pub app_name: String,
    pub category: String,
    pub seconds: i64,
    pub percentage: f64,
}

#[derive(Debug, Serialize, Clone)]
pub struct Totals {
    pub keypresses: i64,
    pub left_clicks: i64,
    pub right_clicks: i64,
    pub middle_clicks: i64,
    pub mouse_feet: f64,
    pub controller_buttons: i64,
}

#[derive(Debug, Serialize, Clone)]
pub struct TimelinePoint {
    pub hour_bucket: String,
    pub keypresses: i64,
    pub left_clicks: i64,
    pub right_clicks: i64,
    pub middle_clicks: i64,
    pub mouse_feet: f64,
    pub controller_buttons: i64,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WebHistoryVisit {
    pub id: Option<String>,
    pub url: String,
    pub normalized_url: Option<String>,
    pub host: Option<String>,
    pub title: Option<String>,
    pub visited_at_ms: Option<i64>,
    pub date: Option<String>,
    pub time: Option<String>,
    pub date_time: Option<String>,
    pub timezone: Option<String>,
    pub incognito: Option<bool>,
    pub context: Option<String>,
    pub screenshot_data_uri: Option<String>,
    pub screenshot_mime: Option<String>,
    pub screenshot_captured_at_ms: Option<i64>,
    pub screenshot_status: Option<String>,
    pub tab_id: Option<i64>,
    pub window_id: Option<i64>,
    pub source_event: Option<String>,
    pub created_at_ms: Option<i64>,
    pub updated_at_ms: Option<i64>,
}

#[derive(Debug, Serialize, Clone)]
pub struct MostVisitedWebsite {
    pub host: String,
    pub visits: i64,
}

#[derive(Debug, Serialize, Clone)]
pub struct WebHistoryStatus {
    pub api_connected: bool,
    pub total_visits: i64,
    pub latest_visit_at_ms: Option<i64>,
}

fn host_from_url(url: &str) -> String {
    let without_scheme = url.split_once("://").map(|(_, rest)| rest).unwrap_or(url);
    without_scheme
        .split(['/', '?', '#'])
        .next()
        .unwrap_or("")
        .split('@')
        .last()
        .unwrap_or("")
        .split(':')
        .next()
        .unwrap_or("")
        .trim()
        .to_lowercase()
}

fn display_host(host: &str) -> String {
    let host = host.trim().trim_end_matches('.').to_lowercase();
    host.strip_prefix("www.").unwrap_or(&host).to_string()
}

fn should_ignore_web_history_host(host: &str) -> bool {
    let host = host.trim().trim_end_matches('.').to_lowercase();
    if host.is_empty() {
        return false;
    }

    let host_without_port = host.split(':').next().unwrap_or(&host);
    matches!(
        host_without_port,
        "localhost"
            | "127.0.0.1"
            | "::1"
            | "0.0.0.0"
            | "[::1]"
    ) || host_without_port.ends_with(".localhost")
}

fn normalize_timestamp_ms(timestamp: i64) -> i64 {
    if timestamp > 0 && timestamp < 100_000_000_000 {
        timestamp * 1000
    } else {
        timestamp
    }
}

#[cfg(test)]
mod tests {
    use super::should_ignore_web_history_host;

    #[test]
    fn ignores_localhost_hosts_for_most_visited_search() {
        assert!(should_ignore_web_history_host("localhost"));
        assert!(should_ignore_web_history_host("LOCALHOST"));
        assert!(should_ignore_web_history_host("foo.localhost"));
    }
}

fn table_columns(conn: &SqliteConnection, table: &str) -> rusqlite::Result<HashSet<String>> {
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({})", table))?;
    let columns = stmt
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<Result<HashSet<_>, _>>()?;
    Ok(columns)
}

fn add_column_if_missing(
    conn: &SqliteConnection,
    columns: &mut HashSet<String>,
    table: &str,
    column: &str,
    definition: &str,
) -> rusqlite::Result<()> {
    if !columns.contains(column) {
        conn.execute(
            &format!("ALTER TABLE {} ADD COLUMN {} {}", table, column, definition),
            [],
        )?;
        columns.insert(column.to_string());
    }
    Ok(())
}

fn ensure_web_history_compat(conn: &SqliteConnection) -> rusqlite::Result<()> {
    let mut columns = table_columns(conn, "web_history")?;
    add_column_if_missing(conn, &mut columns, "web_history", "id", "TEXT")?;
    add_column_if_missing(conn, &mut columns, "web_history", "url", "TEXT")?;
    add_column_if_missing(conn, &mut columns, "web_history", "normalized_url", "TEXT")?;
    add_column_if_missing(conn, &mut columns, "web_history", "host", "TEXT")?;
    add_column_if_missing(conn, &mut columns, "web_history", "title", "TEXT")?;
    add_column_if_missing(conn, &mut columns, "web_history", "visited_at_ms", "INTEGER")?;
    add_column_if_missing(conn, &mut columns, "web_history", "date", "TEXT")?;
    add_column_if_missing(conn, &mut columns, "web_history", "time", "TEXT")?;
    add_column_if_missing(conn, &mut columns, "web_history", "date_time", "TEXT")?;
    add_column_if_missing(conn, &mut columns, "web_history", "timezone", "TEXT")?;
    add_column_if_missing(conn, &mut columns, "web_history", "incognito", "INTEGER DEFAULT 0")?;
    add_column_if_missing(conn, &mut columns, "web_history", "context", "TEXT")?;
    add_column_if_missing(conn, &mut columns, "web_history", "screenshot_data_uri", "TEXT")?;
    add_column_if_missing(conn, &mut columns, "web_history", "screenshot_mime", "TEXT")?;
    add_column_if_missing(
        conn,
        &mut columns,
        "web_history",
        "screenshot_captured_at_ms",
        "INTEGER",
    )?;
    add_column_if_missing(conn, &mut columns, "web_history", "screenshot_status", "TEXT")?;
    add_column_if_missing(conn, &mut columns, "web_history", "tab_id", "INTEGER")?;
    add_column_if_missing(conn, &mut columns, "web_history", "window_id", "INTEGER")?;
    add_column_if_missing(conn, &mut columns, "web_history", "source_event", "TEXT")?;
    add_column_if_missing(conn, &mut columns, "web_history", "created_at_ms", "INTEGER")?;
    add_column_if_missing(conn, &mut columns, "web_history", "updated_at_ms", "INTEGER")?;

    Ok(())
}

fn coalesce_existing_column(columns: &HashSet<String>, candidates: &[&str]) -> String {
    let available = candidates
        .iter()
        .filter(|column| columns.contains(**column))
        .copied()
        .collect::<Vec<_>>();

    match available.as_slice() {
        [] => "NULL".to_string(),
        [column] => (*column).to_string(),
        _ => format!("COALESCE({})", available.join(", ")),
    }
}

impl Database {
    /// Open (or create) the SQLite database and run migrations.
    pub async fn open(path: &str) -> Result<Self, tokio_rusqlite::Error> {
        let conn = Connection::open(path).await?;

        conn.call(|conn| {
            conn.execute_batch(
                "
                PRAGMA journal_mode = WAL;
                PRAGMA synchronous = NORMAL;
                PRAGMA busy_timeout = 5000;

                CREATE TABLE IF NOT EXISTS input_events (
                    id            INTEGER PRIMARY KEY AUTOINCREMENT,
                    event_type    TEXT    NOT NULL,
                    timestamp     INTEGER NOT NULL
                );

                CREATE TABLE IF NOT EXISTS mouse_movement (
                    id            INTEGER PRIMARY KEY AUTOINCREMENT,
                    delta_px      REAL    NOT NULL,
                    timestamp     INTEGER NOT NULL
                );

                CREATE TABLE IF NOT EXISTS window_snapshots (
                    id            INTEGER PRIMARY KEY AUTOINCREMENT,
                    app_name      TEXT    NOT NULL,
                    window_title  TEXT,
                    category      TEXT    NOT NULL,
                    timestamp     INTEGER NOT NULL
                );

                CREATE TABLE IF NOT EXISTS hourly_stats (
                    id            INTEGER PRIMARY KEY AUTOINCREMENT,
                    hour_bucket   TEXT    NOT NULL UNIQUE,
                    keypresses    INTEGER DEFAULT 0,
                    left_clicks   INTEGER DEFAULT 0,
                    right_clicks  INTEGER DEFAULT 0,
                    middle_clicks INTEGER DEFAULT 0,
                    mouse_feet    REAL    DEFAULT 0.0,
                    controller_buttons INTEGER DEFAULT 0
                );

                CREATE TABLE IF NOT EXISTS daily_app_time (
                    id            INTEGER PRIMARY KEY AUTOINCREMENT,
                    date_bucket   TEXT    NOT NULL,
                    app_name      TEXT    NOT NULL,
                    category      TEXT    NOT NULL,
                    seconds       INTEGER DEFAULT 0,
                    UNIQUE(date_bucket, app_name)
                );

                CREATE INDEX IF NOT EXISTS idx_input_events_ts ON input_events(timestamp);
                CREATE INDEX IF NOT EXISTS idx_mouse_movement_ts ON mouse_movement(timestamp);
                CREATE INDEX IF NOT EXISTS idx_window_snapshots_ts ON window_snapshots(timestamp);
                CREATE INDEX IF NOT EXISTS idx_hourly_stats_bucket ON hourly_stats(hour_bucket);
                CREATE INDEX IF NOT EXISTS idx_daily_app_time_date ON daily_app_time(date_bucket);

                CREATE TABLE IF NOT EXISTS character_stats (
                    id            INTEGER PRIMARY KEY AUTOINCREMENT,
                    hour_bucket   TEXT    NOT NULL,
                    character     TEXT    NOT NULL,
                    count         INTEGER DEFAULT 0,
                    UNIQUE(hour_bucket, character)
                );
                CREATE INDEX IF NOT EXISTS idx_char_stats_bucket ON character_stats(hour_bucket);

                CREATE TABLE IF NOT EXISTS web_history (
                    id                         TEXT PRIMARY KEY,
                    url                        TEXT    NOT NULL,
                    normalized_url             TEXT,
                    host                       TEXT    NOT NULL,
                    title                      TEXT,
                    visited_at_ms              INTEGER NOT NULL,
                    date                       TEXT,
                    time                       TEXT,
                    date_time                  TEXT,
                    timezone                   TEXT,
                    incognito                  INTEGER NOT NULL DEFAULT 0,
                    context                    TEXT,
                    screenshot_data_uri        TEXT,
                    screenshot_mime            TEXT,
                    screenshot_captured_at_ms  INTEGER,
                    screenshot_status          TEXT,
                    tab_id                     INTEGER,
                    window_id                  INTEGER,
                    source_event               TEXT,
                    created_at_ms              INTEGER,
                    updated_at_ms              INTEGER
                );
                CREATE INDEX IF NOT EXISTS idx_web_history_visited_at ON web_history(visited_at_ms);
                CREATE INDEX IF NOT EXISTS idx_web_history_host ON web_history(host);
                ",
            )?;
            ensure_web_history_compat(conn)?;

            // Migration: add controller_buttons column if it doesn't exist (for existing DBs)
            let _ = conn.execute(
                "ALTER TABLE hourly_stats ADD COLUMN controller_buttons INTEGER DEFAULT 0",
                [],
            ); // Ignore error if column already exists

            Ok(())
        })
        .await?;

        Ok(Self {
            conn: Arc::new(conn),
        })
    }

    /// Upsert website visits sent by the browser extension.
    pub async fn upsert_web_history(
        &self,
        visits: Vec<WebHistoryVisit>,
    ) -> Result<usize, tokio_rusqlite::Error> {
        self.conn
            .call(move |conn| {
                let mut inserted = 0usize;
                let mut stmt = conn.prepare(
                    "INSERT OR REPLACE INTO web_history (
                        id, url, normalized_url, host, title, visited_at_ms, date, time, date_time,
                        timezone, incognito, context, screenshot_data_uri, screenshot_mime,
                        screenshot_captured_at_ms, tab_id, window_id, screenshot_status,
                        source_event, created_at_ms, updated_at_ms
                     )
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21)",
                )?;

                for visit in visits {
                    if visit.url.trim().is_empty() {
                        continue;
                    }

                    let visited_at_ms = visit.visited_at_ms.unwrap_or_else(|| Utc::now().timestamp_millis());
                    let id = visit.id.unwrap_or_else(|| {
                        format!(
                            "{}:{}",
                            visited_at_ms,
                            visit.url
                        )
                    });
                    let host = visit
                        .host
                        .filter(|host| !host.trim().is_empty())
                        .unwrap_or_else(|| host_from_url(&visit.url));

                    if host.is_empty() {
                        continue;
                    }

                    let incognito = if visit.incognito.unwrap_or(false) { 1 } else { 0 };
                    let now = Utc::now().timestamp_millis();
                    let created_at_ms = visit.created_at_ms.unwrap_or(now);
                    let updated_at_ms = visit.updated_at_ms.unwrap_or(now);

                    stmt.execute(params![
                        id,
                        visit.url,
                        visit.normalized_url,
                        host,
                        visit.title,
                        visited_at_ms,
                        visit.date,
                        visit.time,
                        visit.date_time,
                        visit.timezone,
                        incognito,
                        visit.context,
                        visit.screenshot_data_uri,
                        visit.screenshot_mime,
                        visit.screenshot_captured_at_ms,
                        visit.tab_id,
                        visit.window_id,
                        visit.screenshot_status,
                        visit.source_event,
                        created_at_ms,
                        updated_at_ms,
                    ])?;
                    inserted += 1;
                }

                Ok(inserted)
            })
            .await
    }

    /// Query the most visited website for the requested time range.
    pub async fn query_most_visited_website(
        &self,
        hours_back: u32,
    ) -> Result<Option<MostVisitedWebsite>, tokio_rusqlite::Error> {
        self.conn
            .call(move |conn| {
                ensure_web_history_compat(conn)?;
                let columns = table_columns(conn, "web_history")?;
                let timestamp_expr = coalesce_existing_column(
                    &columns,
                    &[
                        "visited_at_ms",
                        "visitedAtMs",
                        "timestamp",
                        "created_at_ms",
                        "createdAtMs",
                    ],
                );

                let cutoff = Utc::now() - chrono::Duration::hours(hours_back as i64);
                let cutoff_ms = cutoff.timestamp_millis();

                let sql = format!(
                    "SELECT host, url, {timestamp_expr} as visit_ts FROM web_history WHERE {timestamp_expr} IS NOT NULL"
                );
                let mut stmt = conn.prepare(&sql)?;
                let rows = stmt.query_map([], |row| {
                    Ok((
                        row.get::<_, Option<String>>(0)?,
                        row.get::<_, Option<String>>(1)?,
                        row.get::<_, Option<i64>>(2)?,
                    ))
                })?;

                let mut counts: HashMap<String, i64> = HashMap::new();
                for row in rows {
                    let (host, url, timestamp) = row?;
                    let Some(timestamp) = timestamp else {
                        continue;
                    };

                    if normalize_timestamp_ms(timestamp) < cutoff_ms {
                        continue;
                    }

                    let source_host = host
                        .filter(|host| !host.trim().is_empty())
                        .or_else(|| url.map(|url| host_from_url(&url)))
                        .unwrap_or_default();
                    let host = display_host(&source_host);
                    if host.is_empty() || should_ignore_web_history_host(&source_host) {
                        continue;
                    }

                    *counts.entry(host).or_insert(0) += 1;
                }

                Ok(counts
                    .into_iter()
                    .max_by(|(left_host, left_count), (right_host, right_count)| {
                        left_count
                            .cmp(right_count)
                            .then_with(|| right_host.cmp(left_host))
                    })
                    .map(|(host, visits)| MostVisitedWebsite { host, visits }))
            })
            .await
    }

    /// Query local web history ingestion status.
    pub async fn query_web_history_status(&self) -> Result<WebHistoryStatus, tokio_rusqlite::Error> {
        self.conn
            .call(move |conn| {
                ensure_web_history_compat(conn)?;
                let columns = table_columns(conn, "web_history")?;
                let timestamp_expr = coalesce_existing_column(
                    &columns,
                    &[
                        "visited_at_ms",
                        "visitedAtMs",
                        "timestamp",
                        "created_at_ms",
                        "createdAtMs",
                    ],
                );

                let total_visits: i64 =
                    conn.query_row("SELECT COUNT(*) FROM web_history", [], |row| row.get(0))?;
                let latest_visit_at_ms: Option<i64> = conn.query_row(
                    &format!("SELECT MAX({timestamp_expr}) FROM web_history"),
                    [],
                    |row| row.get::<_, Option<i64>>(0),
                )?;

                Ok(WebHistoryStatus {
                    api_connected: true,
                    total_visits,
                    latest_visit_at_ms: latest_visit_at_ms.map(normalize_timestamp_ms),
                })
            })
            .await
    }

    /// Insert a raw input event (key, left_click, right_click, middle_click).
    pub async fn insert_input_event(&self, event_type: &str) -> Result<(), tokio_rusqlite::Error> {
        let event_type = event_type.to_string();
        self.conn
            .call(move |conn| {
                let ts = Utc::now().timestamp_millis();
                conn.execute(
                    "INSERT INTO input_events (event_type, timestamp) VALUES (?1, ?2)",
                    params![event_type, ts],
                )?;
                Ok(())
            })
            .await
    }

    /// Insert accumulated mouse delta pixels.
    pub async fn insert_mouse_delta(&self, delta_px: f64) -> Result<(), tokio_rusqlite::Error> {
        self.conn
            .call(move |conn| {
                let ts = Utc::now().timestamp_millis();
                conn.execute(
                    "INSERT INTO mouse_movement (delta_px, timestamp) VALUES (?1, ?2)",
                    params![delta_px, ts],
                )?;
                Ok(())
            })
            .await
    }

    /// Upsert an hourly stats bucket. `field` must be one of:
    /// keypresses, left_clicks, right_clicks, middle_clicks, mouse_feet
    pub async fn upsert_hourly_stats(
        &self,
        bucket: &str,
        field: &str,
        increment: f64,
    ) -> Result<(), tokio_rusqlite::Error> {
        let bucket = bucket.to_string();
        let field = field.to_string();
        self.conn
            .call(move |conn| {
                // First ensure the row exists
                conn.execute(
                    "INSERT OR IGNORE INTO hourly_stats (hour_bucket) VALUES (?1)",
                    params![bucket],
                )?;
                // Then update the specific field
                let sql = format!(
                    "UPDATE hourly_stats SET {} = {} + ?1 WHERE hour_bucket = ?2",
                    field, field
                );
                conn.execute(&sql, params![increment, bucket])?;
                Ok(())
            })
            .await
    }

    /// Upsert daily app time tracking.
    pub async fn upsert_daily_app_time(
        &self,
        date: &str,
        app_name: &str,
        category: &str,
        add_seconds: i64,
    ) -> Result<(), tokio_rusqlite::Error> {
        let date = date.to_string();
        let app_name = app_name.to_string();
        let category = category.to_string();
        self.conn
            .call(move |conn| {
                conn.execute(
                    "INSERT INTO daily_app_time (date_bucket, app_name, category, seconds)
                     VALUES (?1, ?2, ?3, ?4)
                     ON CONFLICT(date_bucket, app_name) DO UPDATE SET
                       seconds = seconds + ?4,
                       category = ?3",
                    params![date, app_name, category, add_seconds],
                )?;
                Ok(())
            })
            .await
    }

    /// Insert a window snapshot.
    pub async fn insert_window_snapshot(
        &self,
        app_name: &str,
        window_title: Option<&str>,
        category: &str,
    ) -> Result<(), tokio_rusqlite::Error> {
        let app_name = app_name.to_string();
        let window_title = window_title.map(|s| s.to_string());
        let category = category.to_string();
        self.conn
            .call(move |conn| {
                let ts = Utc::now().timestamp_millis();
                conn.execute(
                    "INSERT INTO window_snapshots (app_name, window_title, category, timestamp)
                     VALUES (?1, ?2, ?3, ?4)",
                    params![app_name, window_title, category, ts],
                )?;
                Ok(())
            })
            .await
    }

    /// Query hourly stats for the past N hours.
    #[allow(dead_code)]
    pub async fn query_hourly_stats(
        &self,
        hours_back: u32,
    ) -> Result<Vec<HourlyStat>, tokio_rusqlite::Error> {
        self.conn
            .call(move |conn| {
                let cutoff = Utc::now() - chrono::Duration::hours(hours_back as i64);
                let cutoff_str = cutoff.format("%Y-%m-%dT%H:00").to_string();
                let mut stmt = conn.prepare(
                    "SELECT hour_bucket, keypresses, left_clicks, right_clicks, middle_clicks, mouse_feet, controller_buttons
                     FROM hourly_stats
                     WHERE hour_bucket >= ?1
                     ORDER BY hour_bucket ASC",
                )?;
                let rows = stmt
                    .query_map(params![cutoff_str], |row| {
                        Ok(HourlyStat {
                            hour_bucket: row.get(0)?,
                            keypresses: row.get(1)?,
                            left_clicks: row.get(2)?,
                            right_clicks: row.get(3)?,
                            middle_clicks: row.get(4)?,
                            mouse_feet: row.get(5)?,
                            controller_buttons: row.get(6)?,
                        })
                    })?
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(rows)
            })
            .await
    }

    /// Query app distribution for the past N days.
    pub async fn query_app_distribution(
        &self,
        days_back: u32,
    ) -> Result<Vec<AppShare>, tokio_rusqlite::Error> {
        self.conn
            .call(move |conn| {
                let cutoff = Utc::now() - chrono::Duration::days(days_back as i64);
                let cutoff_str = cutoff.format("%Y-%m-%d").to_string();
                let mut stmt = conn.prepare(
                    "SELECT app_name, category, SUM(seconds) as total_seconds
                     FROM daily_app_time
                     WHERE date_bucket >= ?1
                     GROUP BY app_name
                     ORDER BY total_seconds DESC",
                )?;
                let rows: Vec<(String, String, i64)> = stmt
                    .query_map(params![cutoff_str], |row| {
                        Ok((row.get(0)?, row.get(1)?, row.get(2)?))
                    })?
                    .collect::<Result<Vec<_>, _>>()?;

                let total_secs: i64 = rows.iter().map(|(_, _, s)| s).sum();
                let total = if total_secs > 0 {
                    total_secs as f64
                } else {
                    1.0
                };

                let result: Vec<AppShare> = rows
                    .into_iter()
                    .map(|(app_name, category, seconds)| {
                        let percentage = ((seconds as f64 / total) * 10000.0).round() / 100.0;
                        AppShare {
                            app_name,
                            category,
                            seconds,
                            percentage,
                        }
                    })
                    .collect();

                Ok(result)
            })
            .await
    }

    /// Query totals for the requested time range.
    pub async fn query_totals_range(
        &self,
        hours_back: u32,
    ) -> Result<Totals, tokio_rusqlite::Error> {
        self.conn
            .call(move |conn| {
                let cutoff = Utc::now() - chrono::Duration::hours(hours_back as i64);
                let cutoff_str = cutoff.format("%Y-%m-%dT%H:00").to_string();
                let mut stmt = conn.prepare(
                    "SELECT
                       COALESCE(SUM(keypresses), 0),
                       COALESCE(SUM(left_clicks), 0),
                       COALESCE(SUM(right_clicks), 0),
                       COALESCE(SUM(middle_clicks), 0),
                       COALESCE(SUM(mouse_feet), 0.0),
                       COALESCE(SUM(controller_buttons), 0)
                     FROM hourly_stats
                     WHERE hour_bucket >= ?1",
                )?;
                let totals = stmt.query_row(params![cutoff_str], |row| {
                    Ok(Totals {
                        keypresses: row.get(0)?,
                        left_clicks: row.get(1)?,
                        right_clicks: row.get(2)?,
                        middle_clicks: row.get(3)?,
                        mouse_feet: row.get(4)?,
                        controller_buttons: row.get(5)?,
                    })
                })?;
                Ok(totals)
            })
            .await
    }

    /// Query the past time range timeline, aggregated hourly for 24h and daily otherwise.
    pub async fn query_timeline_range(
        &self,
        hours_back: u32,
    ) -> Result<Vec<TimelinePoint>, tokio_rusqlite::Error> {
        if hours_back <= 24 {
            return self.query_hourly_timeline(hours_back).await;
        }

        self.query_daily_timeline((hours_back + 23) / 24).await
    }

    async fn query_hourly_timeline(
        &self,
        hours_back: u32,
    ) -> Result<Vec<TimelinePoint>, tokio_rusqlite::Error> {
        self.conn
            .call(move |conn| {
                let cutoff = Utc::now() - chrono::Duration::hours(hours_back as i64);
                let cutoff_str = cutoff.format("%Y-%m-%dT%H:00").to_string();
                let mut stmt = conn.prepare(
                    "SELECT hour_bucket, keypresses, left_clicks, right_clicks,
                            middle_clicks, mouse_feet, controller_buttons
                     FROM hourly_stats
                     WHERE hour_bucket >= ?1
                     ORDER BY hour_bucket ASC",
                )?;
                let rows = stmt
                    .query_map(params![cutoff_str], |row| {
                        Ok(TimelinePoint {
                            hour_bucket: row.get(0)?,
                            keypresses: row.get(1)?,
                            left_clicks: row.get(2)?,
                            right_clicks: row.get(3)?,
                            middle_clicks: row.get(4)?,
                            mouse_feet: row.get(5)?,
                            controller_buttons: row.get(6)?,
                        })
                    })?
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(rows)
            })
            .await
    }

    async fn query_daily_timeline(
        &self,
        days_back: u32,
    ) -> Result<Vec<TimelinePoint>, tokio_rusqlite::Error> {
        self.conn
            .call(move |conn| {
                let cutoff = Utc::now() - chrono::Duration::days(days_back as i64);
                let cutoff_str = cutoff.format("%Y-%m-%d").to_string();
                let mut stmt = conn.prepare(
                    "SELECT substr(hour_bucket, 1, 10) as day_bucket,
                            SUM(keypresses), SUM(left_clicks), SUM(right_clicks),
                            SUM(middle_clicks), SUM(mouse_feet), SUM(controller_buttons)
                     FROM hourly_stats
                     WHERE hour_bucket >= ?1
                     GROUP BY day_bucket
                     ORDER BY day_bucket ASC",
                )?;
                let rows = stmt
                    .query_map(params![cutoff_str], |row| {
                        Ok(TimelinePoint {
                            hour_bucket: row.get(0)?,
                            keypresses: row.get(1)?,
                            left_clicks: row.get(2)?,
                            right_clicks: row.get(3)?,
                            middle_clicks: row.get(4)?,
                            mouse_feet: row.get(5)?,
                            controller_buttons: row.get(6)?,
                        })
                    })?
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(rows)
            })
            .await
    }

    /// Query daily totals for the requested time range.
    pub async fn query_daily_stats(
        &self,
        days_back: u32,
    ) -> Result<Vec<DailyStat>, tokio_rusqlite::Error> {
        self.conn
            .call(move |conn| {
                let cutoff = Utc::now() - chrono::Duration::days(days_back as i64);
                let cutoff_str = cutoff.format("%Y-%m-%d").to_string();
                let mut stmt = conn.prepare(
                    "SELECT substr(hour_bucket, 1, 10) as day_date,
                            SUM(keypresses), SUM(left_clicks), SUM(right_clicks),
                            SUM(middle_clicks), SUM(mouse_feet), SUM(controller_buttons)
                     FROM hourly_stats
                     WHERE hour_bucket >= ?1
                     GROUP BY day_date
                     ORDER BY day_date DESC",
                )?;
                let rows = stmt
                    .query_map(params![cutoff_str], |row| {
                        Ok(DailyStat {
                            day: row.get(0)?,
                            keypresses: row.get(1)?,
                            left_clicks: row.get(2)?,
                            right_clicks: row.get(3)?,
                            middle_clicks: row.get(4)?,
                            mouse_feet: row.get(5)?,
                            controller_buttons: row.get(6)?,
                        })
                    })?
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(rows)
            })
            .await
    }

    /// Upsert character stats for a given hour bucket.
    pub async fn upsert_character_stats(
        &self,
        bucket: &str,
        character: &str,
        increment: i64,
    ) -> Result<(), tokio_rusqlite::Error> {
        let bucket = bucket.to_string();
        let character = character.to_string();
        self.conn
            .call(move |conn| {
                conn.execute(
                    "INSERT INTO character_stats (hour_bucket, character, count)
                     VALUES (?1, ?2, ?3)
                     ON CONFLICT(hour_bucket, character) DO UPDATE SET
                       count = count + ?3",
                    params![bucket, character, increment],
                )?;
                Ok(())
            })
            .await
    }

    /// Query aggregated character stats.
    pub async fn query_character_stats(
        &self,
        hours_back: u32,
    ) -> Result<Vec<CharacterStat>, tokio_rusqlite::Error> {
        self.conn
            .call(move |conn| {
                let cutoff = Utc::now() - chrono::Duration::hours(hours_back as i64);
                let cutoff_str = cutoff.format("%Y-%m-%dT%H:00").to_string();
                let mut stmt = conn.prepare(
                    "SELECT character, SUM(count) as total_count
                     FROM character_stats
                     WHERE hour_bucket >= ?1
                     GROUP BY character
                     ORDER BY total_count DESC",
                )?;
                let rows = stmt
                    .query_map(params![cutoff_str], |row| {
                        Ok(CharacterStat {
                            character: row.get(0)?,
                            count: row.get(1)?,
                        })
                    })?
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(rows)
            })
            .await
    }
}
