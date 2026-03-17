# TraceAtlas — Setup Guide

---

## Prerequisites

Install all of these before proceeding.

### 1. Node.js (v18+)
Download from [nodejs.org](https://nodejs.org)

Verify:
```bash
node --version   # v18.x or higher
npm --version
```

### 2. Rust
Install from [rustup.rs](https://rustup.rs):
```bash
# Windows — run the installer from rustup.rs
# macOS / Linux
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Verify:
```bash
rustc --version   # rustc 1.7x.x
cargo --version
```

### 3. WebView2 (Windows only)
Pre-installed on Windows 10 (v1803+) and Windows 11. No action needed.

If missing: download from [Microsoft WebView2](https://developer.microsoft.com/en-us/microsoft-edge/webview2/)

### 4. Microsoft C++ Build Tools (Windows only)
Required for compiling Rust crates with native components.

Download from [visualstudio.microsoft.com/visual-cpp-build-tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)

Select: **Desktop development with C++**

---

## Installation

### 1. Install JS dependencies
```bash
cd TraceAtlas_v6_1
npm install
```

### 2. Install Rust dependencies
Cargo automatically downloads Rust crates on first build. No manual step needed.

---

## Updating the App Icon

Icon files are committed to the repo (`src-tauri/icons/`). If you replace `src-tauri/icons/icon.svg` with a new logo, regenerate all sizes with:
```bash
npm run icons
```
This produces PNG, ICO, ICNS, and platform-specific variants (Windows APPX tiles, iOS, Android) from the single SVG source.

---

## Running in Development

```bash
npm run tauri dev
```

This:
1. Starts the Vite dev server (hot-reload) on `http://localhost:5173`
2. Compiles the Rust layer
3. Opens the app window

First run takes 2–5 minutes (Rust crates compile). Subsequent runs are fast.

---

## Building for Production

```bash
npm run tauri build
```

Output: `src-tauri/target/release/bundle/`

| Platform | Output | Size |
|----------|--------|------|
| Windows | `.msi` installer + `.exe` | ~10–15 MB |
| macOS | `.dmg` + `.app` | ~10–15 MB |
| Linux | `.deb` + `.AppImage` | ~10–15 MB |

---

## Submarine Cable Data

The cable GeoJSON (708 cables from TeleGeography) is already included at:
```
public/data/cables.geojson
```

To refresh it with the latest data:
```bash
# Requires Node.js, run from project root
node -e "
  fetch('https://www.submarinecablemap.com/api/v3/cable/cable-geo.json')
    .then(r => r.json())
    .then(d => require('fs').writeFileSync('public/data/cables.geojson', JSON.stringify(d)))
    .then(() => console.log('Updated'))
"
```

---

## Database

SQLite is managed automatically by Tauri. The database file is stored at:

| Platform | Location |
|----------|----------|
| Windows | `%APPDATA%\com.traceatlas.app\traceatlas.db` |
| macOS | `~/Library/Application Support/com.traceatlas.app/traceatlas.db` |
| Linux | `~/.local/share/com.traceatlas.app/traceatlas.db` |

Tables created automatically on first launch:
- `ip_cache` — geo results, 30-day TTL
- `trace_cache` — full trace results, 1-hour TTL

To reset the cache: delete the `.db` file and relaunch.

---

## Troubleshooting

### App window doesn't open
- Check that Rust compiled successfully (look for errors in terminal)
- Ensure WebView2 is installed (Windows)

### Map tiles not loading
- Check internet connection (tiles served by OpenStreetMap)
- Some corporate firewalls block tile servers

### Traceroute returns no hops
- Windows: `tracert` is a Windows built-in, always available
- Linux/macOS: install `traceroute` if missing (`apt install traceroute` / `brew install traceroute`)
- Some networks block ICMP — results may be partial (`* * *`)

### Geo lookup fails / all hops show unknown
- ip-api.com may be temporarily down or rate-limited
- Check internet connection
- Previously looked-up IPs will still resolve from SQLite cache

### First build is very slow
- Normal — Rust crates compile from source on first build
- Subsequent builds only recompile changed files

### `cargo not found`
- Rust is not installed or not in PATH
- Run `rustup update` or reinstall from rustup.rs
- Restart terminal after installation

---

## Project Structure Reference

```
TraceAtlas_v6_1/
├── src-tauri/          ← Rust / Tauri config
│   ├── Cargo.toml      ← Rust dependencies
│   ├── tauri.conf.json ← app config (name, window size, etc.)
│   ├── capabilities/   ← permission declarations
│   └── src/
│       ├── main.rs     ← app entry point
│       └── lib.rs      ← run_traceroute Rust command
│
├── src/                ← Vue 3 frontend
│   ├── lib/            ← JS business logic
│   │   ├── db.js       ← SQLite helpers
│   │   ├── geo.js      ← IP geolocation + caching
│   │   ├── traceroute.js ← output parsing
│   │   └── insights.js ← insights engine
│   └── components/     ← Vue components
│
├── public/data/
│   └── cables.geojson  ← submarine cable overlay data
│
├── index.html
├── vite.config.js
└── package.json
```
