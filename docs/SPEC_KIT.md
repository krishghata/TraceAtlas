# TraceAtlas — Technical Specification

---

## Metadata

| Field | Value |
|-------|-------|
| Name | TraceAtlas |
| Type | Desktop Application |
| Version | 1.0.0 |
| Owner | Krishnendu Ghata |
| Status | Implemented |
| Framework | Tauri 2 + Vue 3 + Leaflet |

---

## Problem Statement

Internet routing is invisible to users. Tools like `traceroute` provide raw textual output that is difficult to understand and visualize.

TraceAtlas transforms this data into an **interactive, visual, and educational desktop experience** — running real traceroute on the user's own machine so the path shown is accurate to their network.

---

## Why Desktop (Not Web)?

A web app running traceroute on a cloud server shows the *server's* path, not the user's. Browser APIs cannot run raw socket operations (ICMP, TTL manipulation) regardless of technology — including WebAssembly, WebRTC, or WebSockets. The only architecturally correct solution without requiring a local agent is a desktop app.

Tauri is used instead of Electron because:
- No bundled Chromium — uses the OS WebView (WebView2 on Windows, WebKit on macOS)
- Installer size: ~10–15 MB vs ~150 MB
- RAM usage: ~30–60 MB vs ~200+ MB
- The only Rust code needed is ~30 lines to shell out traceroute

---

## Architecture

### High-Level Flow

```
User enters domain/IP
        ↓
Vue.js calls invoke('run_traceroute', target)
        ↓
Rust executes tracert/traceroute on user's machine
        ↓
Raw output returned to JavaScript
        ↓
JS: parse → filter private IPs → deduplicate
        ↓
JS: batch geo-enrich via ip-api.com (with SQLite cache)
        ↓
JS: generate insights (rule-based)
        ↓
Vue: render map + animation + panels
```

### Layer Responsibilities

**Rust layer (`src-tauri/src/lib.rs`)**
- Execute `tracert` (Windows) or `traceroute` (Linux/macOS) via `std::process::Command`
- Validate input (alphanumeric + dot + hyphen only) to prevent injection
- Return raw stdout + `is_win` boolean

**JavaScript lib layer (`src/lib/`)**
- `traceroute.js` — parse raw output, filter private IPs, deduplicate hops
- `geo.js` — batch IP geolocation with SQLite cache
- `insights.js` — rule-based insight generation
- `db.js` — SQLite access via `@tauri-apps/plugin-sql`

**Vue layer (`src/components/`)**
- All UI rendering, map, animation, panels

---

## API: Tauri IPC Command

### `run_traceroute(target: String) → TracerouteResult`

**Input validation:** target must match `/^[a-zA-Z0-9.\-]+$/`

**Response:**
```typescript
{
  output: string,   // raw traceroute stdout
  is_win: boolean   // true on Windows (affects parsing)
}
```

---

## Data Flow Detail

### 1. Input Validation
- Rust validates target before executing any OS command
- Regex: alphanumeric, dot, hyphen only — prevents shell injection

### 2. Trace Cache Check
- SQLite `trace_cache` checked first (1-hour TTL)
- Cache hit → return immediately, no traceroute executed

### 3. Traceroute Execution
- Windows: `tracert -d -h 20 -w 3000 <target>`
- Linux/macOS: `traceroute -n -m 20 -w 3 <target>`
- Flags: skip DNS (`-d`/`-n`), max 20 hops, 3s timeout per hop

### 4. Output Parsing
- Regex-based line-by-line parsing
- Extracts: hop number, IP address, average RTT latency
- `* * *` (timeout) lines skipped

### 5. Private IP Filtering
Removed ranges:
- `10.0.0.0/8`
- `172.16.0.0/12`
- `192.168.0.0/16`
- `127.0.0.0/8`

### 6. Deduplication
- Consecutive hops with the same IP removed (keeps first occurrence)

### 7. Geo Enrichment
See Geo Strategy section below.

### 8. Insights Generation
See Insights Engine section below.

### 9. Cache & Return
- Full result stored in `trace_cache` (1-hour TTL)
- Result returned to Vue for rendering

---

## Geo Strategy

### Primary: SQLite Cache
- Table: `ip_cache` (ip, lat, lon, country, org, updated_at)
- TTL: 30 days
- Persists across app sessions (stored in OS app-data folder)
- Fresh cache hit → returned immediately, no API call

### Fallback: ip-api.com Batch API
Called when: cache miss OR cache entry older than 30 days

- Endpoint: `POST http://ip-api.com/batch`
- Fields: `query, lat, lon, countryCode, org, status`
- Up to 100 IPs per request (entire trace in one call)
- Free tier: 45 requests/minute
- Results stored in SQLite immediately

### Last Resort
If ip-api.com fails:
- Use stale SQLite cache entry
- Return null (hop excluded from map)

### Source Location
Desktop app: `GET http://ip-api.com/json` (no IP argument) returns the machine's own public IP and location — this IS the user since the app runs locally.

### Note on geoip-lite
`geoip-lite` (MaxMind GeoLite2) was used in the previous web architecture but is Node.js-only and cannot run in the Tauri WebView renderer process. The SQLite persistent cache fulfils the same role of "local fast lookup" for previously seen IPs.

---

## Insights Engine

All insights are rule-based. No external AI or ML.

| Insight | Logic |
|---------|-------|
| **Story** | Template string summarising countries, hop count, total distance |
| **Country transitions** | Consecutive hops with different `country` field |
| **Total distance** | Haversine sum across all consecutive hop pairs |
| **Distance fun fact** | Threshold-based: > 15,000 km / > 5,000 km / > 1,000 km |
| **Latency spike** | Consecutive hop latency increase > 50ms |
| **Bottleneck** | Hop with maximum latency across the trace |
| **Ocean crossing** | Country change + haversine distance > 3,000 km between hops |

---

## Data Model (SQLite)

### ip_cache

| Field | Type | Notes |
|-------|------|-------|
| ip | TEXT PK | IPv4 address |
| lat | REAL | Latitude |
| lon | REAL | Longitude |
| country | TEXT | ISO 3166-1 alpha-2 code |
| org | TEXT | Organisation / ISP name |
| updated_at | INTEGER | Unix timestamp (ms) |

### trace_cache

| Field | Type | Notes |
|-------|------|-------|
| target | TEXT PK | Domain or IP queried |
| result | TEXT | Full JSON response |
| updated_at | INTEGER | Unix timestamp (ms) |

---

## Hop Data Model

```typescript
{
  hop:     number,          // hop sequence number
  ip:      string,          // public IPv4 address
  lat:     number,          // latitude
  lon:     number,          // longitude
  country: string,          // ISO country code (e.g. "US")
  org:     string | null,   // organisation / ASN name
  latency: number | null    // average RTT in ms
}
```

---

## Map & Visualization

### Tile Layer
- OpenStreetMap tiles
- CSS filter: `brightness(0.7) saturate(0.6) hue-rotate(200deg)` → dark blue theme

### Route Rendering (Glow Arc)
- Arc points generated with sine-based latitude offset scaled by hop distance
- 5 overlapping polylines with decreasing opacity (outer glow → bright core)
- Colors: `#38bdf8` outer, `#bae6fd` core

### Hop Markers (Latency Heatmap)
- Color scale: `rgb(0,210,60)` (fast) → `rgb(255,210,0)` (medium) → `rgb(255,0,40)` (slow)
- Normalized to min/max latency of the trace

### Submarine Cable Overlay
- Source: TeleGeography public API (708 cable systems)
- File: `public/data/cables.geojson`
- Toggle: shown/hidden via checkbox
- Style: `#f97316`, weight 1.2, opacity 0.45

### Animation
- Auto-play: packet marker steps through coords at 500ms intervals
- Step-by-step: ◀ ▶ buttons for manual control
- Replay: restarts auto-play from hop 1

---

## Security

| Concern | Mitigation |
|---------|-----------|
| Command injection | Input validated in Rust before `Command::new()` — args passed as array, never shell-interpolated |
| Private IP leakage | Private/loopback IPs stripped before geo-enrichment |
| SQLite injection | Parameterised queries throughout (`?` placeholders) |
| API rate limit | SQLite cache minimises ip-api.com calls; stale cache used as fallback |

---

## Tech Stack

| Component | Technology |
|-----------|-----------|
| Desktop framework | Tauri 2 |
| Frontend framework | Vue 3 (Composition API) |
| Build tool | Vite 5 |
| Map library | Leaflet 1.9 |
| Screenshot | html2canvas |
| Database | SQLite via `@tauri-apps/plugin-sql` |
| Geo API | ip-api.com (free tier) |
| Cable data | TeleGeography public API |
| OS layer | Rust (`std::process::Command`) |

---

## Success Criteria

- User inputs domain/IP → route visualised within ~10 seconds
- Traceroute path reflects the user's actual network (runs locally)
- Animation plays smoothly
- Insights are meaningful and readable
- Installer < 20 MB
- App RAM usage < 100 MB

---

## Non-Goals

- Exact routing accuracy (IP geolocation is approximate)
- Real-time packet tracking
- Enterprise-grade network diagnostics
- Full IPv6 support
- Multi-user / cloud deployment

---

## Future Scope

| Area | Feature |
|------|---------|
| Visualization | 3D globe (WebGL / Three.js) |
| Visualization | Latency heatmap over time |
| Intelligence | RIPE Atlas probe integration |
| Intelligence | ASN / ISP graph visualization |
| Intelligence | Route anomaly detection |
| Intelligence | BGP dataset integration |
| UX | Timeline view |
| UX | Export/share route as link |
| Data | Real-time cable dataset updates |

---

## Philosophy

> TraceAtlas is not about perfect accuracy.
> It is about making the invisible internet **visible, understandable, and engaging**.
