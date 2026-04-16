# StealthMon — Silent Activity Monitor for Windows

A production-ready, always-on background activity monitor that runs silently in the
Windows system tray. It captures keyboard, mouse, and application usage data, stores
everything locally in a SQLite database, and serves a beautiful terminal-green web
dashboard at `http://localhost:9521`.

## Features

- **Silent background operation** — no console window, just a system tray icon
- **Global input tracking** — counts keypresses, left/right/middle clicks (never logs actual keys)
- **Mouse distance** — tracks how far your mouse travels in feet
- **Active window monitoring** — polls every 5s, categorises apps automatically
- **Local web dashboard** — Chart.js-powered dark terminal UI with live data
- **Privacy-first** — no key content stored, banking/password apps redacted, all data local
- **Auto-refreshing dashboard** — updates every 60 seconds

## What Data Is Stored

All data is kept in a local SQLite database at `%APPDATA%\ActivityMonitor\activity.db`:

| Table | What it stores |
|-------|---------------|
| `input_events` | Event type (key/click) + timestamp. **Never the actual key.** |
| `mouse_movement` | Accumulated pixel deltas (converted to feet) |
| `window_snapshots` | App name, window title, category, timestamp |
| `hourly_stats` | Aggregated counts per hour bucket |
| `daily_app_time` | Seconds spent per app per day |

**Privacy protections:**
- Actual keystrokes are **never recorded** — only a counter is incremented
- Password managers and banking apps are stored as `"private"` with `NULL` window titles
- The web server only binds to `127.0.0.1` (localhost) — never exposed to the network

## Building

### Prerequisites

- [Rust](https://rustup.rs/) (latest stable, 1.75+)
- Windows 10/11

### Build Release

```bash
cargo build --release
```

The compiled binary will be at `target/release/stealthmon.exe`.

### Build Debug

```bash
cargo build
```

## Running

Simply double-click `stealthmon.exe` or run it from the command line. It will:

1. Create `%APPDATA%\ActivityMonitor\` directory (if it doesn't exist)
2. Initialize the SQLite database
3. Start all background collectors
4. Start the web dashboard server on `http://localhost:9521`
5. Place a green icon in the system tray

### System Tray Menu

Right-click the tray icon for options:

- **Open Dashboard** — launches `http://localhost:9521` in your default browser
- **Quit** — gracefully shuts down all collectors and flushes pending data

## Opening the Dashboard

Either:
- Right-click the tray icon → "Open Dashboard"
- Open your browser and go to `http://localhost:9521`

The dashboard shows:
- **Donut chart** — top apps by usage time
- **Radar chart** — time distribution by category
- **Stat cards** — total clicks, keypresses, mouse movement (in feet)
- **24-hour timeline** — multi-line chart of all metrics
- **Weekly averages** — table showing average activity per weekday

## Auto-Start on Windows Login

To have StealthMon start automatically when you log in, add it to the Windows
registry Run key:

### Option 1: Registry Editor (GUI)

1. Press `Win+R`, type `regedit`, press Enter
2. Navigate to: `HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run`
3. Right-click → New → String Value
4. Name: `StealthMon`
5. Value: full path to your exe, e.g. `C:\path\to\stealthmon.exe`

### Option 2: PowerShell (one-liner)

```powershell
New-ItemProperty -Path "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run" `
  -Name "StealthMon" `
  -Value "C:\path\to\stealthmon.exe" `
  -PropertyType String -Force
```

Replace `C:\path\to\stealthmon.exe` with the actual path.

### Option 3: Startup Folder

1. Press `Win+R`, type `shell:startup`, press Enter
2. Create a shortcut to `stealthmon.exe` in the folder that opens

## Application Categories

Apps are automatically categorised:

| Category | Apps |
|----------|------|
| Coding | VS Code, Zed, Cursor, Neovide, NeoVim, JetBrains IDEs, Sublime Text, Notepad++ |
| Gaming | Steam, Epic, Minecraft, Valorant, League of Legends, CS2, Overwatch, Fortnite, Roblox |
| Browser | Chrome, Firefox, Edge, Opera, Brave, Vivaldi, Arc |
| Communication | Discord, Slack, Teams, Telegram, WhatsApp, Signal, Zoom, Skype |
| Media | MPV, VLC, Spotify, Netflix, YouTube Music, Crunchyroll |
| Creative | Blender, Figma, Photoshop, Illustrator, DaVinci, Premiere, After Effects, Krita, GIMP |
| Productivity | Word, Excel, PowerPoint, Notion, Obsidian, OneNote, LibreOffice |
| Other | Anything not matched above |

## Logs

Application logs are written to `%APPDATA%\ActivityMonitor\app.log`.

## License

MIT