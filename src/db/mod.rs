use chrono::Utc;
use rusqlite::params;
use serde::Serialize;
use std::sync::Arc;
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
                ",
            )?;

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
                        let percentage =
                            ((seconds as f64 / total) * 10000.0).round() / 100.0;
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

