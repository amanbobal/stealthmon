use reqwest::Client;
use semver::Version;
use serde::{Deserialize, Serialize};
use std::{
    path::{Path, PathBuf},
    process::Command,
    sync::Arc,
};
use tokio::sync::Mutex;

const REPO_NAME: &str = "stealthmon";
const RELEASES_LATEST_URL: &str =
    "https://api.github.com/repos/amanbobal/stealthmon/releases/latest";

#[derive(Debug, Clone, Serialize)]
pub struct UpdateStatus {
    pub current_version: String,
    pub latest_version: Option<String>,
    pub update_available: bool,
    pub release_url: Option<String>,
    pub asset_name: Option<String>,
    pub asset_size: Option<u64>,
    pub checked_at: Option<String>,
    pub status: String,
    pub error: Option<String>,
    pub update_in_progress: bool,
}

#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    html_url: String,
    assets: Vec<GitHubAsset>,
}

#[derive(Debug, Deserialize, Clone)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
    size: u64,
}

#[derive(Clone)]
pub struct UpdateManager {
    client: Client,
    data_dir: PathBuf,
    state: Arc<Mutex<UpdateStatus>>,
}

impl UpdateManager {
    pub fn new(data_dir: PathBuf) -> Self {
        let current_version = current_version().to_string();
        Self {
            client: Client::new(),
            data_dir,
            state: Arc::new(Mutex::new(UpdateStatus {
                current_version,
                latest_version: None,
                update_available: false,
                release_url: None,
                asset_name: None,
                asset_size: None,
                checked_at: None,
                status: "not_checked".to_string(),
                error: None,
                update_in_progress: false,
            })),
        }
    }

    pub async fn status(&self) -> UpdateStatus {
        self.state.lock().await.clone()
    }

    pub async fn check_for_updates(&self) -> UpdateStatus {
        {
            let mut state = self.state.lock().await;
            state.status = "checking".to_string();
            state.error = None;
        }

        let next_state = match self.fetch_latest_release().await {
            Ok((release, asset)) => build_checked_status(release, asset),
            Err(error) => {
                let mut state = self.state.lock().await.clone();
                state.status = "check_failed".to_string();
                state.error = Some(error);
                state.checked_at = Some(chrono::Utc::now().to_rfc3339());
                state
            }
        };

        let mut state = self.state.lock().await;
        *state = next_state.clone();
        next_state
    }

    pub async fn install_update(&self) -> Result<UpdateStatus, String> {
        {
            let mut state = self.state.lock().await;
            if state.update_in_progress {
                return Ok(state.clone());
            }
            state.update_in_progress = true;
            state.status = "downloading".to_string();
            state.error = None;
        }

        let result = self.download_and_schedule_update().await;
        match result {
            Ok(status) => Ok(status),
            Err(error) => {
                let mut state = self.state.lock().await;
                state.update_in_progress = false;
                state.status = "install_failed".to_string();
                state.error = Some(error.clone());
                Err(error)
            }
        }
    }

    async fn fetch_latest_release(&self) -> Result<(GitHubRelease, GitHubAsset), String> {
        let release = self
            .client
            .get(RELEASES_LATEST_URL)
            .header("User-Agent", format!("{REPO_NAME}/{}", current_version()))
            .header("Accept", "application/vnd.github+json")
            .send()
            .await
            .map_err(|error| format!("GitHub release check failed: {error}"))?
            .error_for_status()
            .map_err(|error| format!("GitHub release check failed: {error}"))?
            .json::<GitHubRelease>()
            .await
            .map_err(|error| format!("Invalid GitHub release response: {error}"))?;

        let asset = release
            .assets
            .iter()
            .find(|asset| {
                let name = asset.name.to_lowercase();
                name.ends_with(".exe")
                    && (name.contains("windows")
                        || name.contains("win64")
                        || name.contains("x64")
                        || name.contains("x86_64"))
            })
            .or_else(|| {
                release
                    .assets
                    .iter()
                    .find(|asset| asset.name.to_lowercase().ends_with(".exe"))
            })
            .cloned()
            .ok_or_else(|| "Latest release does not include a Windows .exe asset".to_string())?;

        Ok((release, asset))
    }

    async fn download_and_schedule_update(&self) -> Result<UpdateStatus, String> {
        let (release, asset) = self.fetch_latest_release().await?;
        let latest = parse_release_version(&release.tag_name)?;
        let current = parse_release_version(current_version())?;
        if latest <= current {
            let status = build_checked_status(release, asset);
            let mut state = self.state.lock().await;
            *state = status.clone();
            return Ok(status);
        }

        let update_dir = self.data_dir.join("updates");
        tokio::fs::create_dir_all(&update_dir)
            .await
            .map_err(|error| format!("Failed to create update directory: {error}"))?;

        let download_path = update_dir.join(format!("stealthmon-{}.exe", latest));
        let bytes = self
            .client
            .get(&asset.browser_download_url)
            .header("User-Agent", format!("{REPO_NAME}/{}", current_version()))
            .send()
            .await
            .map_err(|error| format!("Update download failed: {error}"))?
            .error_for_status()
            .map_err(|error| format!("Update download failed: {error}"))?
            .bytes()
            .await
            .map_err(|error| format!("Failed to read update download: {error}"))?;

        tokio::fs::write(&download_path, bytes)
            .await
            .map_err(|error| format!("Failed to save update: {error}"))?;

        schedule_replace_current_exe(&update_dir, &download_path)?;

        let mut status = build_checked_status(release, asset);
        status.status = "restart_scheduled".to_string();
        status.update_in_progress = true;

        let mut state = self.state.lock().await;
        *state = status.clone();

        std::thread::spawn(|| {
            std::thread::sleep(std::time::Duration::from_millis(700));
            std::process::exit(0);
        });

        Ok(status)
    }
}

pub async fn auto_check_loop(manager: UpdateManager, cancel: tokio_util::sync::CancellationToken) {
    // Initial check
    let status = manager.check_for_updates().await;
    if status.update_available {
        use windows_sys::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONINFORMATION, MB_YESNO};
        let title: Vec<u16> = "StealthMon Update Available\0".encode_utf16().collect();
        let msg_text = format!("New version {} is available. Install now?", status.latest_version.unwrap_or_default());
        let msg: Vec<u16> = format!("{}\0", msg_text).encode_utf16().collect();
        unsafe {
            let res = MessageBoxW(0, msg.as_ptr(), title.as_ptr(), MB_YESNO | MB_ICONINFORMATION);
            const IDYES: i32 = 6;
            if res == IDYES {
                let _ = manager.install_update().await;
            }
        }
    }

    let mut interval = tokio::time::interval(std::time::Duration::from_secs(6 * 60 * 60));

    loop {
        tokio::select! {
            _ = cancel.cancelled() => break,
            _ = interval.tick() => {
                let status = manager.check_for_updates().await;
                if status.update_available {
                    use windows_sys::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONINFORMATION, MB_YESNO};
                    let title: Vec<u16> = "StealthMon Update Available\0".encode_utf16().collect();
                    let msg_text = format!("New version {} is available. Install now?", status.latest_version.unwrap_or_default());
                    let msg: Vec<u16> = format!("{}\0", msg_text).encode_utf16().collect();
                    unsafe {
                        let res = MessageBoxW(0, msg.as_ptr(), title.as_ptr(), MB_YESNO | MB_ICONINFORMATION);
                        const IDYES: i32 = 6;
                        if res == IDYES {
                            let _ = manager.install_update().await;
                        }
                    }
                }
            }
        }
    }
}

fn build_checked_status(release: GitHubRelease, asset: GitHubAsset) -> UpdateStatus {
    let latest_version = parse_release_version(&release.tag_name).ok();
    let current = parse_release_version(current_version()).ok();
    let update_available = match (&latest_version, &current) {
        (Some(latest), Some(current)) => latest > current,
        _ => false,
    };

    UpdateStatus {
        current_version: current_version().to_string(),
        latest_version: latest_version.map(|version| version.to_string()),
        update_available,
        release_url: Some(release.html_url),
        asset_name: Some(asset.name),
        asset_size: Some(asset.size),
        checked_at: Some(chrono::Utc::now().to_rfc3339()),
        status: "checked".to_string(),
        error: None,
        update_in_progress: false,
    }
}

fn current_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

fn parse_release_version(value: &str) -> Result<Version, String> {
    Version::parse(value.trim().trim_start_matches('v'))
        .map_err(|error| format!("Invalid release version '{value}': {error}"))
}

fn schedule_replace_current_exe(update_dir: &Path, download_path: &Path) -> Result<(), String> {
    let current_exe =
        std::env::current_exe().map_err(|error| format!("Failed to locate current exe: {error}"))?;
    let script_path = update_dir.join("apply-stealthmon-update.ps1");
    let pid = std::process::id();
    let script = r#"
param(
  [int]$TargetPid,
  [string]$NewExe,
  [string]$TargetExe
)

$deadline = (Get-Date).AddSeconds(30)
while ((Get-Process -Id $TargetPid -ErrorAction SilentlyContinue) -and ((Get-Date) -lt $deadline)) {
  Start-Sleep -Milliseconds 250
}
Start-Sleep -Milliseconds 500
Copy-Item -LiteralPath $NewExe -Destination $TargetExe -Force
Start-Process -FilePath $TargetExe
Remove-Item -LiteralPath $NewExe -Force -ErrorAction SilentlyContinue
Remove-Item -LiteralPath $MyInvocation.MyCommand.Path -Force -ErrorAction SilentlyContinue
"#;

    std::fs::write(&script_path, script)
        .map_err(|error| format!("Failed to write update script: {error}"))?;

    Command::new("powershell.exe")
        .arg("-NoProfile")
        .arg("-ExecutionPolicy")
        .arg("Bypass")
        .arg("-WindowStyle")
        .arg("Hidden")
        .arg("-File")
        .arg(&script_path)
        .arg("-TargetPid")
        .arg(pid.to_string())
        .arg("-NewExe")
        .arg(download_path)
        .arg("-TargetExe")
        .arg(current_exe)
        .spawn()
        .map_err(|error| format!("Failed to start update handoff: {error}"))?;

    Ok(())
}
