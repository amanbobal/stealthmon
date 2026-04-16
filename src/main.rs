#![windows_subsystem = "windows"]

mod collectors;
mod db;
mod server;

use db::Database;
use std::path::PathBuf;
use tokio_util::sync::CancellationToken;
use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
    TrayIconBuilder, Icon,
};

fn get_data_dir() -> PathBuf {
    let appdata = std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
    let dir = PathBuf::from(appdata).join("ActivityMonitor");
    std::fs::create_dir_all(&dir).ok();
    dir
}

fn setup_logging(data_dir: &PathBuf) {
    use tracing_subscriber::{fmt, EnvFilter};

    let log_path = data_dir.join("app.log");
    let file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .expect("Failed to open log file");

    fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_writer(std::sync::Mutex::new(file))
        .with_ansi(false)
        .init();
}

fn load_tray_icon() -> Icon {
    // Try to load from the embedded ICO, fall back to a generated icon
    let icon_bytes = include_bytes!("../assets/tray_icon.ico");
    
    // Parse ICO file - try to use image crate
    match image::load_from_memory(icon_bytes) {
        Ok(img) => {
            let rgba = img.to_rgba8();
            let (w, h) = (rgba.width(), rgba.height());
            Icon::from_rgba(rgba.into_raw(), w, h).unwrap_or_else(|_| create_fallback_icon())
        }
        Err(_) => create_fallback_icon(),
    }
}

fn create_fallback_icon() -> Icon {
    // Create a simple 32x32 green square icon
    let size = 32u32;
    let mut rgba = vec![0u8; (size * size * 4) as usize];
    for y in 0..size {
        for x in 0..size {
            let idx = ((y * size + x) * 4) as usize;
            // Green (#00ff88) with full alpha
            rgba[idx] = 0x00;     // R
            rgba[idx + 1] = 0xff; // G
            rgba[idx + 2] = 0x88; // B
            rgba[idx + 3] = 0xff; // A
        }
    }
    Icon::from_rgba(rgba, size, size).expect("Failed to create fallback icon")
}

fn main() {
    let data_dir = get_data_dir();
    setup_logging(&data_dir);

    tracing::info!("StealthMon starting up");
    tracing::info!("Data directory: {}", data_dir.display());

    let db_path = data_dir.join("activity.db");
    let db_path_str = db_path.to_string_lossy().to_string();

    // Build the tokio runtime
    let runtime = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");

    let cancel = CancellationToken::new();

    // Initialize DB and spawn all async tasks
    let db = runtime.block_on(async {
        Database::open(&db_path_str)
            .await
            .expect("Failed to open database")
    });

    tracing::info!("Database initialized at {}", db_path.display());

    // Spawn input collector (rdev hook in OS thread + async processor)
    let rx = collectors::input::start_input_collector();

    let cancel_input = cancel.clone();
    let db_input = db.clone();
    runtime.spawn(async move {
        tracing::info!("Input processor task starting");
        collectors::input::process_input_events(db_input, rx, cancel_input).await;
        tracing::info!("Input processor task ended");
    });

    // Spawn window polling task
    let db_window = db.clone();
    let cancel_window = cancel.clone();
    runtime.spawn(async move {
        tracing::info!("Window collector task starting");
        collectors::window::start_window_collector(db_window, cancel_window).await;
        tracing::info!("Window collector task ended");
    });

    // Spawn HTTP server (with panic catching)
    let db_server = db.clone();
    let cancel_server = cancel.clone();
    runtime.spawn(async move {
        tracing::info!("HTTP server task starting");
        server::start_server(db_server, cancel_server).await;
        tracing::info!("HTTP server task ended");
    });

    // Give tasks a moment to start before entering the blocking tray loop
    std::thread::sleep(std::time::Duration::from_millis(200));
    tracing::info!("All background tasks spawned, entering tray icon event loop");

    // ── System tray setup (must be on main thread) ──
    let icon = load_tray_icon();

    let menu = Menu::new();
    let open_dashboard = MenuItem::new("Open Dashboard", true, None);
    let separator = PredefinedMenuItem::separator();
    let quit_item = MenuItem::new("Quit", true, None);

    menu.append(&open_dashboard).ok();
    menu.append(&separator).ok();
    menu.append(&quit_item).ok();

    let open_id = open_dashboard.id().clone();
    let quit_id = quit_item.id().clone();

    let _tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip("StealthMon — Activity Monitor")
        .with_icon(icon)
        .build()
        .expect("Failed to create tray icon");

    // Tray event loop — uses a simple polling loop
    let menu_rx = MenuEvent::receiver();

    // Spawn background thread to poll menu events
    std::thread::spawn(move || {
        loop {
            if let Ok(event) = menu_rx.try_recv() {
                if event.id == open_id {
                    tracing::info!("Opening dashboard in browser");
                    if let Err(e) = open::that("http://localhost:9521") {
                        tracing::error!("Failed to open browser: {}", e);
                    }
                } else if event.id == quit_id {
                    tracing::info!("Quit requested from tray menu");
                    cancel.cancel();
                    // Give tasks a moment to flush
                    std::thread::sleep(std::time::Duration::from_millis(500));
                    unsafe {
                        windows_sys::Win32::UI::WindowsAndMessaging::PostQuitMessage(0);
                    }
                    break;
                }
            }
            // Sleep briefly to avoid busy-spinning
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    });

    // Main thread MUST run the Windows message loop, otherwise the tray icon freezes
    unsafe {
        use windows_sys::Win32::Foundation::HWND;
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            DispatchMessageW, GetMessageW, TranslateMessage, MSG,
        };
        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, 0, 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }

    tracing::info!("StealthMon shutting down");
    // Drop runtime to clean up
    drop(runtime);
}
