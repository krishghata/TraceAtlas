# TraceAtlas

**Visualize how your internet traffic travels from your machine to any destination**

TraceAtlas is a desktop application that runs real `traceroute` on your machine, geo-enriches every hop, and renders the path as an interactive animated map — with insights, submarine cable overlays, and latency heatmaps.

---

## Why Desktop?

Traceroute must run on the user's own machine to show the *actual* path their traffic takes. A server-side approach would show the server's path, not yours. TraceAtlas is built with **Tauri** — a Rust + WebView framework that gives a native desktop experience with a ~10–15 MB installer (no bundled Chromium).

---

## Features

### Map & Visualization
- Global Leaflet map with dark tile theme
- Animated packet movement along the route (auto-play + step-by-step)
- Curved arc paths between hops (arc height scales with distance)
- Glow polyline rendering (multi-layer neon effect)
- Auto-zoom to fit the full route
- Latency heatmap — hop markers colored green → yellow → red by latency
- Submarine cable overlay (708 real cables from TeleGeography, toggle on/off)

### Insights Engine (rule-based, no AI API)
- Story — plain-English summary of the route
- Country transitions — every border crossing detected
- Total distance — haversine-calculated with fun facts
- Latency spike detection — flags jumps > 50ms
- Bottleneck — identifies the single slowest hop
- Ocean crossing heuristic — country change + distance > 3,000 km

### UI Panels
- **Summary bar** — hops, countries, distance, max latency, source/destination IPs
- **Hop list** — per-hop: number, country, IP, org, latency (color-coded)
- **Insights panel** — all insight cards
- Step-by-step hop playback (◀ ▶ buttons)
- Replay animation
- Export trace as JSON
- Screenshot capture (PNG)

### Data & Caching
- SQLite database persisted in OS app-data folder
- `ip_cache` — geo results cached 30 days, avoids repeated API calls
- `trace_cache` — full trace results cached 1 hour
- Geo: ip-api.com batch API (free, 45 req/min) with SQLite as local cache

---

## Architecture

```
┌─────────────────────────────────────────┐
│  Vue 3 + Leaflet (WebView renderer)     │
│  All logic runs here as JavaScript:     │
│  • traceroute output parsing            │
│  • geo enrichment (ip-api.com + SQLite) │
│  • insights generation                  │
│  • map rendering + animation            │
├─────────────────────────────────────────┤
│  Tauri Rust layer (thin OS bridge)      │
│  • run_traceroute command               │
│    → executes tracert/traceroute        │
│    → returns raw output + platform flag │
│  • @tauri-apps/plugin-sql (SQLite)      │
├─────────────────────────────────────────┤
│  OS WebView2 (Windows) / WebKit (macOS) │
│  Pre-installed — not bundled            │
└─────────────────────────────────────────┘
```

---

## Project Structure

```
TraceAtlas_v6_1/
├── src-tauri/                  ← Rust / Tauri config
│   ├── Cargo.toml
│   ├── build.rs
│   ├── tauri.conf.json
│   ├── capabilities/
│   │   └── default.json
│   └── src/
│       ├── main.rs             ← entry point
│       └── lib.rs              ← run_traceroute command (~30 lines)
│
├── src/                        ← Vue 3 frontend
│   ├── lib/                    ← business logic (JS)
│   │   ├── db.js               ← SQLite via Tauri SQL plugin
│   │   ├── geo.js              ← ip-api.com + cache
│   │   ├── traceroute.js       ← parsing, dedup, IP filter
│   │   └── insights.js         ← all insight rules
│   ├── components/
│   │   ├── Landing.vue
│   │   ├── AppView.vue         ← orchestration + toolbar
│   │   ├── MapView.vue         ← Leaflet map + animation
│   │   ├── HopList.vue
│   │   ├── SummaryPanel.vue
│   │   └── InsightsPanel.vue
│   ├── main.js
│   └── App.vue
│
├── public/
│   └── data/
│       └── cables.geojson      ← 708 TeleGeography submarine cables
│
├── docs/
│   ├── SETUP.md
│   ├── SPEC_KIT.md
│   └── PROJECT_NOTES.md
│
├── index.html
├── vite.config.js
└── package.json
```

---

## How It Works

1. User enters a domain or IP and clicks **Trace**
2. Tauri calls `run_traceroute` (Rust) → executes `tracert`/`traceroute` on the user's machine
3. Raw output is parsed in JavaScript → private IPs filtered → duplicates removed
4. All public hop IPs are batch geo-enriched via ip-api.com → results cached in SQLite
5. Insights are generated (rule-based, no AI)
6. Map renders with glow arcs, heatmap markers, and cable overlay
7. Packet animation plays automatically; step-by-step controls available

---

## Quick Start

See [docs/SETUP.md](docs/SETUP.md) for full setup instructions.

```bash
# Install dependencies
npm install

# Dev mode (hot-reload)
npm run tauri dev

# Production build (~10–15 MB installer)
npm run tauri build
```

---

## Limitations

- IP geolocation is approximate (city-level accuracy not guaranteed)
- Some hops timeout or are hidden by firewalls (`* * *`)
- Undersea cable detection is heuristic (country change + distance > 3,000 km)
- Traceroute path is a snapshot — routing changes dynamically
- ip-api.com free tier: 45 requests/minute (SQLite cache keeps actual usage very low)

---

## Future Scope

- RIPE Atlas integration — traceroute from probes worldwide
- 3D globe visualization (WebGL / Three.js)
- ASN / ISP graph
- Route anomaly detection
- Curved Bezier arcs (currently great-circle arc interpolation)
- Real-time latency heatmap
- BGP dataset integration

---

## Philosophy

> TraceAtlas is not about perfect accuracy.
> It is about making the invisible internet **visible, understandable, and engaging**.

---

*Built with Tauri, Vue 3, Leaflet, and ip-api.com*
