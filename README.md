<div align="center">
  <img src="public/logo-light.svg" alt="TraceAtlas Logo" width="120" height="120"/>
  <h1>TraceAtlas</h1>
  <p><strong>Visualize how your internet traffic travels — hop by hop, country by country, across the globe.</strong></p>

  <p>
    <img alt="Tauri" src="https://img.shields.io/badge/Tauri_2-0a0f1e?logo=tauri&logoColor=38bdf8"/>
    <img alt="Vue 3" src="https://img.shields.io/badge/Vue_3-0a0f1e?logo=vue.js&logoColor=38bdf8"/>
    <img alt="Leaflet" src="https://img.shields.io/badge/Leaflet-0a0f1e?logo=leaflet&logoColor=38bdf8"/>
    <img alt="Rust" src="https://img.shields.io/badge/Rust-0a0f1e?logo=rust&logoColor=38bdf8"/>
    <img alt="License" src="https://img.shields.io/badge/License-MIT-0a0f1e?logoColor=38bdf8"/>
  </p>
</div>

---

## Screenshots

> *Screenshots coming soon*

| Landing | Trace in progress | Results |
|---------|------------------|---------|
| ![Landing screen](docs/screenshots/landing.png) | ![Tracing](docs/screenshots/tracing.png) | ![Results map](docs/screenshots/results.png) |

| Ping chart | Network Health | Speed Test |
|------------|---------------|------------|
| ![Ping](docs/screenshots/ping.png) | ![Network Health](docs/screenshots/health.png) | ![Speed Test](docs/screenshots/speedtest.png) |

---

## Why Desktop?

Traceroute must run on **your machine** to show your actual network path. A server-side approach traces the server's route, not yours. TraceAtlas is built with **Tauri** — a Rust + OS WebView framework with a ~10–15 MB installer and no bundled Chromium.

| | Tauri (TraceAtlas) | Electron | Web App |
|--|-------------------|----------|---------|
| Installer size | ~10–15 MB | ~150 MB | N/A |
| RAM usage | ~30–60 MB | ~200 MB | N/A |
| Traceroute accuracy | Your path ✅ | Your path ✅ | Server's path ❌ |
| WebView | OS native | Bundled Chromium | Browser |

---

## Features

### ⤷ Traceroute
- Global Leaflet map with dark tile theme
- Animated packet movement along the route (auto-play + step-by-step)
- Curved arc paths between hops — arc height scales with distance
- Glow polyline rendering (multi-layer neon effect)
- Auto-zoom to fit the full route
- Latency heatmap — hop markers colored green → yellow → red
- **Submarine cable overlay** — 708 real cables from TeleGeography (toggle on/off)
- **Cable Route** — toggle to highlight the nearest matching submarine cable in gold for each long-distance hop (proximity scoring algorithm)
- **Summary bar** — hops, countries, distance, max latency, source/destination IPs
- **Hop list** — per-hop: number, country flag, IP, org, latency (color-coded)
- **Insights panel** — plain-English route story, country transitions, latency spikes, bottleneck detection
- Step-by-step hop playback (◀ ▶), replay animation
- Export trace as JSON
- Screenshot capture (PNG)

### 📡 Ping
- Real-time RTT chart with live SVG line graph and area fill
- **Continuous mode** — streams one packet at a time, updates the chart live
- **Batch mode** — 10, 20, or 50 packets in a single run
- Color-coded dots: green < 50 ms · amber < 150 ms · orange < 300 ms · red / timeout
- Dropped packets shown as × markers on the chart baseline
- Stats bar: Sent · Received · Loss % · Min / Avg / Max / Jitter
- Live pulse indicator while running

### 🩺 Network Health
- Diagnoses your connection **layer by layer** — Local → Router → ISP → Internet → DNS
- Runs all 4 pings concurrently (gateway, 8.8.8.8, 1.1.1.1, google.com)
- **Topology diagram** — animated SVG path from You → Router → ISP → Internet, color-coded by status
- **Layer cards** with sparkline charts and gradient area fills
- **Latency comparison bar chart** — side-by-side horizontal bars for all 4 layers
- **Diagnosis card** — pinpoints the exact problem (local WiFi / ISP / DNS) with a plain-English suggestion
- **Auto-run** — configurable interval (30 s · 1 min · 5 min · 10 min) with countdown timer
- Auto-detects your gateway IP via `ipconfig` / `ip route`

### ⚡ Speed Test
- Measures **download speed**, **upload speed**, and **latency** via Cloudflare's public endpoints
- Progressive measurement — tests increasing file sizes for accuracy
- **Semicircle gauges** — animated arc fill for each metric
- **Live chart** — area graph of download (blue) and upload (purple) speed over time
- Phase progress bar: Ping → Download → Upload → Done
- Latency rated: Excellent / Good / Fair / Poor

### General
- **Tab state preservation** — switching tabs never resets charts or results (Vue `<KeepAlive>`)
- **No console flicker** — all system commands (`tracert`, `ping`, `ipconfig`) run silently without a visible terminal window (`CREATE_NO_WINDOW`)
- **Non-blocking UI** — all Rust commands run on a background thread pool (`spawn_blocking`)
- **Auto-updater** — checks for new releases on startup, installs in-app without re-downloading the installer
- **SQLite cache** — geo results cached 30 days, trace results 1 hour
- **Splash screen** — auto-launches after 3 seconds with a progress bar

---

## How It Works

```
User enters domain / IP
         ↓
Tauri invoke('run_traceroute')  [async, no console window]
         ↓
Rust executes tracert / traceroute on user's machine
         ↓
JS: parse → filter private IPs → deduplicate hops
         ↓
JS: batch geo-enrich via ip-api.com (SQLite cache)
         ↓
JS: generate insights (rule-based)
         ↓
Vue: render map + animation + panels
```

```
Ping tab: invoke('run_ping') in a loop → parse RTT → live SVG chart
Health tab: 4 concurrent invokes → topology + sparklines + diagnosis
Speed test: fetch() to Cloudflare CDN → measure throughput → gauge + chart
```

---

## Quick Start

See [docs/SETUP.md](docs/SETUP.md) for full prerequisites and setup.

```bash
# Install dependencies
npm install

# Dev mode (hot-reload)
npm run tauri dev

# Production build (~10–15 MB installer)
npm run tauri build

# Regenerate app icons from SVG source
npm run icons
```

---

## Architecture

```
┌─────────────────────────────────────────────────────┐
│  Vue 3 + Leaflet (WebView renderer)                 │
│  • Traceroute: parsing, geo-enrichment, map         │
│  • Ping: live RTT chart (SVG)                       │
│  • Network Health: topology diagram, diagnosis      │
│  • Speed Test: Cloudflare fetch, gauge + chart      │
│  • SQLite cache via Tauri SQL plugin                │
├─────────────────────────────────────────────────────┤
│  Tauri Rust layer                                   │
│  • run_traceroute  (async, CREATE_NO_WINDOW)        │
│  • run_ping        (async, CREATE_NO_WINDOW)        │
│  • get_default_gateway (async, CREATE_NO_WINDOW)    │
│  • tauri-plugin-sql (SQLite)                        │
│  • tauri-plugin-updater (auto-update)               │
├─────────────────────────────────────────────────────┤
│  OS WebView — pre-installed, not bundled            │
│  Windows: WebView2   macOS: WebKit                  │
│  Linux: WebKitGTK                                   │
└─────────────────────────────────────────────────────┘
```

---

## Project Structure

```
TraceAtlas/
├── src-tauri/                  ← Rust / Tauri config
│   ├── src/lib.rs              ← run_traceroute, run_ping, get_default_gateway
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── capabilities/default.json
│   └── icons/                  ← app icons (all platforms, generated via npm run icons)
│
├── src/                        ← Vue 3 frontend
│   ├── App.vue                 ← tab bar, KeepAlive, auto-updater banner
│   ├── lib/
│   │   ├── db.js               ← SQLite via Tauri SQL plugin
│   │   ├── geo.js              ← ip-api.com + SQLite cache
│   │   ├── traceroute.js       ← parsing, dedup, IP filter
│   │   ├── insights.js         ← all insight rules
│   │   └── ping.js             ← RTT parsing, color mapping, jitter
│   └── components/
│       ├── Landing.vue         ← splash screen (auto-launches after 3 s)
│       ├── AppView.vue         ← Traceroute tab orchestration + toolbar
│       ├── MapView.vue         ← Leaflet map, animation, cable route highlight
│       ├── HopList.vue
│       ├── SummaryPanel.vue
│       ├── InsightsPanel.vue
│       ├── PingView.vue        ← Ping tab — live RTT chart
│       ├── NetworkHealthView.vue ← Network Health tab — topology + diagnosis
│       └── SpeedTestView.vue   ← Speed Test tab — download/upload gauges
│
├── public/
│   ├── logo.svg                ← dark logo
│   ├── logo-light.svg          ← light logo (used in app + splash)
│   └── data/cables.geojson    ← 708 TeleGeography submarine cables
│
└── docs/
    ├── SETUP.md
    ├── SPEC_KIT.md
    └── PROJECT_NOTES.md
```

---

## Limitations

- IP geolocation is approximate — city-level accuracy not guaranteed
- Some hops timeout or are hidden by firewalls (`* * *`)
- Cable route matching is proximity-based — shows the nearest cable, not the guaranteed path
- Traceroute path is a snapshot — internet routing changes dynamically
- Speed test accuracy depends on network conditions at the time of the test
- Not available on mobile (traceroute requires OS-level command execution)

---

## Future Scope

- RIPE Atlas integration — traceroute from probes worldwide
- 3D globe visualization (WebGL / Three.js)
- ASN / ISP graph
- Route anomaly detection
- Real-time latency heatmap over time
- BGP dataset integration
- Windows code signing (Microsoft Trusted Signing — free for open source)

---

## License

MIT © [Krishnendu Ghata](https://github.com/krishghata)

---

<div align="center">
  <sub>Built with Tauri · Vue 3 · Leaflet · Rust · ip-api.com · Cloudflare Speed Test</sub>
</div>
