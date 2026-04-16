use crate::db::Database;
use chrono::Utc;
use rdev::{listen, Button, EventType};
use std::sync::{
    atomic::{AtomicI64, Ordering},
    Arc,
};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

/// Input event types we track.
#[derive(Debug, Clone)]
pub enum InputEvent {
    Key,
    LeftClick,
    RightClick,
    MiddleClick,
    MouseMove { x: f64, y: f64 },
    ControllerButton,
}

/// Running counters for the current hour bucket.
struct HourlyCounters {
    keypresses: AtomicI64,
    left_clicks: AtomicI64,
    right_clicks: AtomicI64,
    middle_clicks: AtomicI64,
    controller_buttons: AtomicI64,
}

impl HourlyCounters {
    fn new() -> Self {
        Self {
            keypresses: AtomicI64::new(0),
            left_clicks: AtomicI64::new(0),
            right_clicks: AtomicI64::new(0),
            middle_clicks: AtomicI64::new(0),
            controller_buttons: AtomicI64::new(0),
        }
    }

    fn reset(&self) -> (i64, i64, i64, i64, i64) {
        let kp = self.keypresses.swap(0, Ordering::Relaxed);
        let lc = self.left_clicks.swap(0, Ordering::Relaxed);
        let rc = self.right_clicks.swap(0, Ordering::Relaxed);
        let mc = self.middle_clicks.swap(0, Ordering::Relaxed);
        let cb = self.controller_buttons.swap(0, Ordering::Relaxed);
        (kp, lc, rc, mc, cb)
    }
}

/// Start the rdev global hook in a dedicated OS thread and forward events
/// through an mpsc channel to async tasks.
pub fn start_input_collector() -> mpsc::UnboundedReceiver<InputEvent> {
    let (tx, rx) = mpsc::unbounded_channel::<InputEvent>();

    // Spawn the blocking rdev listener in a standard thread
    let tx_rdev = tx.clone();
    std::thread::Builder::new()
        .name("rdev-listener".into())
        .spawn(move || {
            tracing::info!("Starting rdev global input hook");
            if let Err(e) = listen(move |event| {
                let input_event = match event.event_type {
                    EventType::KeyPress(_) => Some(InputEvent::Key),
                    EventType::ButtonPress(Button::Left) => Some(InputEvent::LeftClick),
                    EventType::ButtonPress(Button::Right) => Some(InputEvent::RightClick),
                    EventType::ButtonPress(Button::Middle) => Some(InputEvent::MiddleClick),
                    EventType::MouseMove { x, y } => Some(InputEvent::MouseMove { x, y }),
                    _ => None,
                };
                if let Some(evt) = input_event {
                    let _ = tx_rdev.send(evt);
                }
            }) {
                tracing::error!(
                    "rdev::listen failed (possibly missing permissions): {:?}. \
                     Input collection disabled.",
                    e
                );
            }
        })
        .expect("Failed to spawn rdev listener thread");

    // Spawn a gamepad polling thread using gilrs (xinput backend)
    std::thread::Builder::new()
        .name("gamepad-listener".into())
        .spawn(move || {
            tracing::info!("Starting gamepad input listener");
            match gilrs::Gilrs::new() {
                Ok(mut gilrs) => {
                    for (_id, gamepad) in gilrs.gamepads() {
                        tracing::info!("Gamepad connected: {}", gamepad.name());
                    }
                    loop {
                        while let Some(gilrs::Event { event, .. }) = gilrs.next_event() {
                            if matches!(event, gilrs::EventType::ButtonPressed(_, _)) {
                                let _ = tx.send(InputEvent::ControllerButton);
                            }
                        }
                        // Poll at ~60Hz to avoid busy-spinning
                        std::thread::sleep(std::time::Duration::from_millis(16));
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        "Failed to initialize gamepad input (gilrs): {:?}. \
                         Controller tracking disabled.",
                        e
                    );
                }
            }
        })
        .expect("Failed to spawn gamepad listener thread");

    rx
}

/// Process input events from the channel, writing to DB and maintaining hourly counters.
pub async fn process_input_events(
    db: Database,
    mut rx: mpsc::UnboundedReceiver<InputEvent>,
    cancel: CancellationToken,
) {
    let counters = Arc::new(HourlyCounters::new());
    let counters_flush = counters.clone();
    let db_flush = db.clone();
    let cancel_flush = cancel.clone();

    // Mouse distance accumulator
    let mouse_acc = Arc::new(std::sync::Mutex::new((0.0f64, None::<(f64, f64)>)));
    let mouse_acc_flush = mouse_acc.clone();
    let db_mouse = db.clone();
    let cancel_mouse = cancel.clone();

    // Hourly bucket flush task — check every 5 seconds
    tokio::spawn(async move {
        let mut current_bucket = current_hour_bucket();
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));
        loop {
            tokio::select! {
                _ = cancel_flush.cancelled() => {
                    // Final flush on shutdown
                    flush_counters(&db_flush, &counters_flush, &current_bucket).await;
                    break;
                }
                _ = interval.tick() => {
                    let new_bucket = current_hour_bucket();
                    if new_bucket != current_bucket {
                        // Hour changed — flush old bucket
                        flush_counters(&db_flush, &counters_flush, &current_bucket).await;
                        current_bucket = new_bucket;
                    } else {
                        // Same hour — still flush periodically to not lose data
                        flush_counters(&db_flush, &counters_flush, &current_bucket).await;
                    }
                }
            }
        }
    });

    // Mouse distance flush task — every 5 seconds
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));
        loop {
            tokio::select! {
                _ = cancel_mouse.cancelled() => {
                    // Final mouse flush
                    let delta = {
                        let mut acc = mouse_acc_flush.lock().unwrap();
                        let d = acc.0;
                        acc.0 = 0.0;
                        d
                    };
                    if delta > 0.0 {
                        let feet = delta / 1152.0;
                        let bucket = current_hour_bucket();
                        let _ = db_mouse.insert_mouse_delta(delta).await;
                        let _ = db_mouse.upsert_hourly_stats(&bucket, "mouse_feet", feet).await;
                    }
                    break;
                }
                _ = interval.tick() => {
                    let delta = {
                        let mut acc = mouse_acc_flush.lock().unwrap();
                        let d = acc.0;
                        acc.0 = 0.0;
                        d
                    };
                    if delta > 0.0 {
                        let feet = delta / 1152.0;
                        let bucket = current_hour_bucket();
                        let _ = db_mouse.insert_mouse_delta(delta).await;
                        let _ = db_mouse.upsert_hourly_stats(&bucket, "mouse_feet", feet).await;
                    }
                }
            }
        }
    });

    // Main event processing loop
    loop {
        tokio::select! {
            _ = cancel.cancelled() => {
                tracing::info!("Input event processor shutting down");
                break;
            }
            event = rx.recv() => {
                match event {
                    Some(InputEvent::Key) => {
                        counters.keypresses.fetch_add(1, Ordering::Relaxed);
                        let _ = db.insert_input_event("key").await;
                    }
                    Some(InputEvent::LeftClick) => {
                        counters.left_clicks.fetch_add(1, Ordering::Relaxed);
                        let _ = db.insert_input_event("left_click").await;
                    }
                    Some(InputEvent::RightClick) => {
                        counters.right_clicks.fetch_add(1, Ordering::Relaxed);
                        let _ = db.insert_input_event("right_click").await;
                    }
                    Some(InputEvent::MiddleClick) => {
                        counters.middle_clicks.fetch_add(1, Ordering::Relaxed);
                        let _ = db.insert_input_event("middle_click").await;
                    }
                    Some(InputEvent::ControllerButton) => {
                        counters.controller_buttons.fetch_add(1, Ordering::Relaxed);
                        let _ = db.insert_input_event("controller_button").await;
                    }
                    Some(InputEvent::MouseMove { x, y }) => {
                        let mut acc = mouse_acc.lock().unwrap();
                        if let Some((lx, ly)) = acc.1 {
                            let dx = x - lx;
                            let dy = y - ly;
                            let dist = (dx * dx + dy * dy).sqrt();
                            acc.0 += dist;
                        }
                        acc.1 = Some((x, y));
                    }
                    None => {
                        tracing::warn!("Input event channel closed");
                        break;
                    }
                }
            }
        }
    }
}

async fn flush_counters(db: &Database, counters: &HourlyCounters, bucket: &str) {
    let (kp, lc, rc, mc, cb) = counters.reset();
    if kp > 0 {
        let _ = db
            .upsert_hourly_stats(bucket, "keypresses", kp as f64)
            .await;
    }
    if lc > 0 {
        let _ = db
            .upsert_hourly_stats(bucket, "left_clicks", lc as f64)
            .await;
    }
    if rc > 0 {
        let _ = db
            .upsert_hourly_stats(bucket, "right_clicks", rc as f64)
            .await;
    }
    if mc > 0 {
        let _ = db
            .upsert_hourly_stats(bucket, "middle_clicks", mc as f64)
            .await;
    }
    if cb > 0 {
        let _ = db
            .upsert_hourly_stats(bucket, "controller_buttons", cb as f64)
            .await;
    }
}

fn current_hour_bucket() -> String {
    Utc::now().format("%Y-%m-%dT%H:00").to_string()
}
