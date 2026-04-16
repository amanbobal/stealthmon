use chrono::{Datelike, NaiveDateTime, Utc};
use rusqlite::params;
use serde::Serialize;
use std::sync::Arc;
use tokio_rusqlite::Connection;

/// Async database wrapper for all activity monitor operations.
#[derive(Clone)]
pub struct Database {
    conn: Arc<Connection>,
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
}

#[derive(Debug, Serialize, Clone)]
pub struct DailyAvgStat {
    pub day: String,
    pub left_clicks: f64,
    pub right_clicks: f64,
    pub middle_clicks: f64,
    pub keypresses: f64,
    pub mouse_feet: f64,
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
}

#[derive(Debug, Serialize, Clone)]
pub struct TimelinePoint {
    pub hour_bucket: String,
    pub keypresses: i64,
    pub left_clicks: i64,
    pub right_clicks: i64,
    pub middle_clicks: i64,
    pub mouse_feet: f64,
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
                    mouse_feet    REAL    DEFAULT 0.0
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
                ",
            )?;
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
                    "SELECT hour_bucket, keypresses, left_clicks, right_clicks, middle_clicks, mouse_feet
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
                        })
                    })?
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(rows)
            })
            .await
    }

    /// Query average daily stats per weekday (Sun–Sat).
    pub async fn query_daily_avg_stats(
        &self,
    ) -> Result<Vec<DailyAvgStat>, tokio_rusqlite::Error> {
        self.conn
            .call(|conn| {
                // Aggregate hourly_stats by date, then average per weekday
                let mut stmt = conn.prepare(
                    "SELECT
                       substr(hour_bucket, 1, 10) as day_date,
                       SUM(keypresses) as kp,
                       SUM(left_clicks) as lc,
                       SUM(right_clicks) as rc,
                       SUM(middle_clicks) as mc,
                       SUM(mouse_feet) as mf
                     FROM hourly_stats
                     GROUP BY day_date
                     ORDER BY day_date",
                )?;

                // Collect per-date totals
                struct DayData {
                    date: String,
                    kp: i64,
                    lc: i64,
                    rc: i64,
                    mc: i64,
                    mf: f64,
                }

                let rows: Vec<DayData> = stmt
                    .query_map([], |row| {
                        Ok(DayData {
                            date: row.get(0)?,
                            kp: row.get(1)?,
                            lc: row.get(2)?,
                            rc: row.get(3)?,
                            mc: row.get(4)?,
                            mf: row.get(5)?,
                        })
                    })?
                    .collect::<Result<Vec<_>, _>>()?;

                // Group by weekday (0=Sun, 6=Sat)
                let day_names = ["SUN", "MON", "TUE", "WED", "THU", "FRI", "SAT"];
                let mut sums: [(f64, f64, f64, f64, f64, f64); 7] =
                    [(0.0, 0.0, 0.0, 0.0, 0.0, 0.0); 7];

                for d in &rows {
                    if let Ok(nd) = NaiveDateTime::parse_from_str(
                        &format!("{}T00:00:00", d.date),
                        "%Y-%m-%dT%H:%M:%S",
                    ) {
                        let weekday = nd.weekday().num_days_from_sunday() as usize;
                        sums[weekday].0 += 1.0; // count
                        sums[weekday].1 += d.lc as f64;
                        sums[weekday].2 += d.rc as f64;
                        sums[weekday].3 += d.mc as f64;
                        sums[weekday].4 += d.kp as f64;
                        sums[weekday].5 += d.mf;
                    }
                }

                let result: Vec<DailyAvgStat> = (0..7)
                    .map(|i| {
                        let count = if sums[i].0 > 0.0 { sums[i].0 } else { 1.0 };
                        DailyAvgStat {
                            day: day_names[i].to_string(),
                            left_clicks: (sums[i].1 / count).round(),
                            right_clicks: (sums[i].2 / count).round(),
                            middle_clicks: (sums[i].3 / count).round(),
                            keypresses: (sums[i].4 / count).round(),
                            mouse_feet: ((sums[i].5 / count) * 100.0).round() / 100.0,
                        }
                    })
                    .collect();

                Ok(result)
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

    /// Query all-time totals.
    pub async fn query_totals_all_time(&self) -> Result<Totals, tokio_rusqlite::Error> {
        self.conn
            .call(|conn| {
                let mut stmt = conn.prepare(
                    "SELECT
                       COALESCE(SUM(keypresses), 0),
                       COALESCE(SUM(left_clicks), 0),
                       COALESCE(SUM(right_clicks), 0),
                       COALESCE(SUM(middle_clicks), 0),
                       COALESCE(SUM(mouse_feet), 0.0)
                     FROM hourly_stats",
                )?;
                let totals = stmt.query_row([], |row| {
                    Ok(Totals {
                        keypresses: row.get(0)?,
                        left_clicks: row.get(1)?,
                        right_clicks: row.get(2)?,
                        middle_clicks: row.get(3)?,
                        mouse_feet: row.get(4)?,
                    })
                })?;
                Ok(totals)
            })
            .await
    }

    /// Query the past 24 hours timeline, one point per hour bucket.
    pub async fn query_24h_timeline(
        &self,
    ) -> Result<Vec<TimelinePoint>, tokio_rusqlite::Error> {
        self.conn
            .call(|conn| {
                let cutoff = Utc::now() - chrono::Duration::hours(24);
                let cutoff_str = cutoff.format("%Y-%m-%dT%H:00").to_string();
                let mut stmt = conn.prepare(
                    "SELECT hour_bucket, keypresses, left_clicks, right_clicks,
                            middle_clicks, mouse_feet
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
                        })
                    })?
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(rows)
            })
            .await
    }
}
