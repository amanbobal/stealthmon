# Vault Web Logger

A Chromium Manifest V3 extension that logs visited websites into a local IndexedDB database and exports vault-friendly files for Obsidian and Capacities.

## What It Stores

Each visit record includes:

- `id`: stable dedupe key for imports and repeated exports
- `url`, `normalizedUrl`, `host`, and page `title`
- simple local `date`, `time`, and `dateTime`
- `context`: `normal` or `incognito`
- `screenshotDataUri`: a base64 data URL string when capture succeeds
- `screenshotStatus`: capture state or failure reason

The extension exports `.jsonl`, `.csv`, and `.md`. JSONL is the best canonical "local database" file for import pipelines because it is append-friendly, line-searchable, and each record carries the stable `id` dedupe key. Markdown exports include an image cell using the screenshot data URI so the image can render in Markdown viewers that allow data URI images.

## Browser Limitations

Extensions cannot safely write a normal SQLite/database file directly into an arbitrary folder such as an Obsidian or Capacities vault without a native companion app. This extension stores data locally in the browser profile and lets you download portable files for your vault.

Screenshot capture is also browser-limited:

- capture works for the visible loaded viewport, not a stitched full-page screenshot
- capture requires the tab to be active; background tabs are logged but may show `pending_tab_not_active`
- browser-internal pages like `chrome://...` are not logged
- incognito logging only works after you explicitly allow the extension in incognito/private windows

## Install

1. Open `chrome://extensions` or the equivalent page in Edge/Brave.
2. Enable developer mode.
3. Click **Load unpacked** and select this folder.
4. Open the extension details page and enable **Allow in incognito**.

## Export Every 14 Days

1. Click the extension icon.
2. Open the log.
3. Click **Last 14 days**.
4. Export:
   - **JSONL** for robust import/dedupe workflows
   - **Markdown** for direct vault viewing
   - **CSV** for spreadsheet-style review

Exports are named with the selected date range and mode, for example `vault-web-log-2026-06-02_to_2026-06-15-all-modes.jsonl`. Use `id` as the dedupe key in any import workflow. Re-exporting the same 14-day data produces the same filename pattern and the same record IDs, so downstream imports can overwrite, upsert, or skip duplicates without manual cleanup.
