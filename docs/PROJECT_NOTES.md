# TraceAtlas — Project Notes & Design Decisions

---

## Purpose

This document captures the key assumptions, design decisions, constraints, trade-offs, and future scope for TraceAtlas. It explains the *why* behind architectural choices so future development stays consistent.

---

## Core Assumptions

### 1. IP Geolocation is Approximate
IP → location mapping is not exact. Accuracy varies by ISP, region, and database freshness.

> Approximate location is sufficient for visualization. TraceAtlas is an educational tool, not a network audit tool.

### 2. Traceroute Output is Incomplete
Some hops do not respond (firewalls, ICMP filtering) and return `* * *`. Some hops are hidden entirely.

> Partial data is acceptable and expected. The app visualises what is available.

### 3. Network Path is Dynamic
Internet routing changes constantly. The same query may take a different path each time.

> The visualization represents a snapshot, not a permanent route.

### 4. Desktop = User's Machine
Since TraceAtlas is a Tauri desktop app, the machine running it IS the user. `traceroute` executes locally. The path shown is the user's actual network path.

---

## Design Decisions

### 1. Tauri Instead of Electron

**Why not Electron:**
- Bundles its own Chromium (~80 MB added to installer)
- Higher RAM usage (~200 MB baseline)
- For a developer showcase tool, a 150 MB installer is a poor first impression

**Why Tauri:**
- Uses OS WebView (WebView2 on Windows, pre-installed on Win10/11)
- Installer: ~10–15 MB
- RAM: ~30–60 MB
- The only Rust code needed is one function (~30 lines) to run a shell command
- Vue.js frontend is identical to a web app — no rewrite needed

### 2. Desktop App Instead of Web App

**Why not web:**
- Browser APIs cannot run `traceroute` — no raw socket access, no TTL manipulation
- WebAssembly runs in the same sandbox as JavaScript — same network restrictions apply
- WebRTC (STUN) can detect your public IP but cannot trace routing hops
- A cloud-deployed backend would trace the *server's* path, not the user's

**Decision:** Desktop app is the only architecturally correct approach for user-accurate routing visualization.

### 3. Rust Layer is Minimal (~30 lines)

Rust is used only for what JavaScript cannot do in a browser context: execute OS commands with `std::process::Command`. Everything else — parsing, geo, insights, UI — runs in JavaScript.

This means:
- Developers do not need deep Rust knowledge to contribute
- All business logic is in readable, testable JavaScript
- The Rust/JS boundary is a single well-defined function

### 4. ip-api.com as Geo Provider

**Why not MaxMind GeoLite2:**
- Requires a free account + license key to download the database
- The `.mmdb` binary format requires a native library to read
- In the Tauri WebView renderer context, `geoip-lite` (Node.js) cannot run

**Why ip-api.com:**
- Free, no registration, no API key
- Batch endpoint (up to 100 IPs per request) — entire trace in one API call
- Returns lat, lon, country, org
- Rate limit (45 req/min) is rarely hit due to SQLite caching

**The SQLite cache fills the role of a local database:**
- First lookup: ip-api.com (slight delay)
- All subsequent lookups: SQLite (instant, no API call)
- Cache TTL: 30 days — stale entries refreshed on next use

### 5. All Business Logic in JavaScript (not Rust)

Parsing, geo-enrichment, insights, and caching all run as JavaScript in the Vue renderer. Alternatives considered:

| Approach | Pros | Cons | Decision |
|----------|------|------|----------|
| JS in renderer | Easy to maintain, same language as UI | Cannot call OS APIs | ✅ Used for all logic |
| Rust Tauri commands | Native speed, full OS access | Requires Rust for every feature | ✅ Used only for traceroute |
| Node.js sidecar | Familiar Node.js | Extra process, defeats purpose of Tauri | ❌ Rejected |

### 6. Rule-Based Insights (No AI API)

**Why:**
- Free — no per-call cost
- Fast — instant, no network round-trip
- Predictable — deterministic output
- Offline-capable — insights work without internet

**Rules implemented:**
- Country transitions (compare consecutive hop `country` fields)
- Latency spike detection (diff > 50ms between consecutive hops)
- Ocean crossing heuristic (country change + haversine > 3,000 km)
- Bottleneck detection (max latency hop)
- Distance fun facts (threshold-based descriptions)

### 7. Curved Arc Rendering (Not Bezier)

True Bezier curves require a Leaflet plugin or canvas overlay. The current approach generates interpolated points along a great-circle-like arc using a sine offset scaled by distance. This achieves the visual effect without a dependency, and the "lift" naturally scales with hop distance (short domestic hops have shallow arcs, transoceanic hops have dramatic curves).

### 8. Submarine Cable Data (TeleGeography)

TeleGeography provides their cable map data via a public API. The GeoJSON (708 cables) is downloaded once and bundled as a static file. This avoids runtime API calls for a dataset that changes rarely.

---

## Limitations

### Geo Accuracy
- City-level accuracy is not guaranteed
- IPs may resolve to ISP headquarters or regional PoPs rather than the actual router location
- This is inherent to all IP geolocation systems

### Undersea Cable Detection
- Detection is heuristic only: country change + distance > 3,000 km
- Does not use actual cable route geometry
- May produce false positives (long overland routes) or miss cables on short cross-sea hops

### Latency Interpretation
- RTT values from traceroute include queuing delay, not just propagation
- ICMP deprioritisation by routers makes some hop latencies artificially high
- Latency values are indicative, not precise

### Missing Hops
- `* * *` hops are filtered out entirely
- Consecutive identical IPs are deduplicated
- Private IPs (LAN, ISP internal) are removed
- The resulting route may have gaps

### ip-api.com Dependency
- Offline use will fail for new IPs (previously cached IPs still work)
- Rate limit: 45 requests/minute. Unlikely to be hit for normal single-user use.
- Free tier may have reduced availability during high-traffic periods

---

## Trade-offs

### Accuracy vs Simplicity
Approximate visualization chosen over complex network modeling. The goal is education, not audit.

### Speed vs Completeness
Partial results shown rather than waiting for full trace. `* * *` hops are skipped rather than causing delays.

### Native vs Web
Chose desktop (Tauri) over web for architectural correctness. The traceroute must run on the user's machine.

### Rust vs Node.js
Kept Rust surface minimal. Business logic in JavaScript for maintainability.

---

## Constraints

- Traceroute execution time: 2–30 seconds (depends on target and network)
- ip-api.com rate limit: 45 req/min (mitigated by SQLite cache)
- WebView rendering: Leaflet performance on very long routes (100+ hops) may degrade
- Some corporate/government networks block ICMP entirely

---

## Future Scope

### High Value / Feasible

**RIPE Atlas Integration**
RIPE Atlas has 10,000+ probes worldwide that can execute traceroute on demand. The API is free for educational use. This would allow users to see routing from a probe near their location, useful for comparing paths from different regions.

**Latency Heatmap Over Time**
Run multiple traces and overlay latency differences on the map, showing which hops degrade over time.

**ASN / ISP Graph**
Visualize which autonomous systems the route passes through as a node graph alongside the geographic map.

### Lower Priority

- 3D globe (Three.js) — high visual impact, significant rewrite of MapView
- BGP route data — complex, requires BGP feed access
- Route anomaly detection — requires historical baseline data
- Timeline view — UX redesign

---

## Guiding Principle

> TraceAtlas is not about perfect accuracy.
> It is about making the invisible internet **visible, understandable, and engaging**.

TraceAtlas is designed as:
- A **visual learning tool** for understanding internet infrastructure
- A **network exploration experience** for curious users
- A **developer showcase project** demonstrating Tauri, Vue, Leaflet, and systems thinking
