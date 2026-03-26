<div align="center">
  <img src="public/logo-light.svg" alt="TraceAtlas Logo" width="120" height="120"/>
  <h1>TraceAtlas</h1>
  <p><strong>A complete network diagnostics toolkit — visualize routes, measure speed, inspect certificates, scan your network, and more.</strong></p>

  <p>
    <img alt="Platform" src="https://img.shields.io/badge/Windows%20%7C%20macOS%20%7C%20Linux-0a0f1e?logoColor=38bdf8"/>
    <img alt="License" src="https://img.shields.io/badge/License-MIT-0a0f1e?logoColor=38bdf8"/>
    <img alt="Release" src="https://img.shields.io/github/v/release/krishghata/TraceAtlas?color=0a0f1e&labelColor=0a0f1e&logoColor=38bdf8"/>
  </p>

  [**Download**](https://github.com/krishghata/TraceAtlas/releases/latest) · [Setup Guide](docs/SETUP.md)
</div>

---

## What is TraceAtlas?

TraceAtlas puts a full network diagnostics lab in a single lightweight desktop app. Whether you're troubleshooting a slow connection, auditing a server's security headers, or just curious about where your packets go — everything is one click away, with no command-line knowledge required.

> Because traceroute must run on **your machine** to show your actual network path, TraceAtlas is a native desktop app. Unlike web-based tools, it traces *your* route — not a server's.

---

## Tools

### ⤷ Traceroute — *Visualize your route on a world map*

See every hop your packets take across the globe, plotted on an interactive dark-themed map with animated packet movement.

![Traceroute](docs/screenshots/traceroute.png)

- Animated hop-by-hop packet visualization with curved arc paths
- Latency heatmap — hops colored green → amber → red by RTT
- **Submarine cable overlay** — 708 real cables from TeleGeography; toggle to highlight the nearest matching cable in gold
- Summary bar: hops · countries · distance · max latency · source / destination
- Plain-English insights: route story, country transitions, latency spikes, bottleneck detection
- Export trace as JSON · screenshot as PNG

---

### 📡 Ping — *Measure latency in real time*

Continuous or batch ping with a live chart. Spot packet loss and jitter at a glance.

![Ping](docs/screenshots/ping.png)

- Live SVG line + area chart, color-coded dots (green / amber / orange / red)
- Dropped packets shown as × markers on the baseline
- Stats: Sent · Received · Loss% · Min / Avg / Max / Jitter
- Continuous mode (streams forever) or batch mode (10 / 20 / 50 packets)

---

### 🩺 Network Health — *Diagnose your connection layer by layer*

One click runs a full diagnosis: local → router → ISP → internet → DNS.

![Network Health](docs/screenshots/health.png)

- Parallel checks across 4 layers with sparkline charts per layer
- SVG topology diagram: You → Router → ISP → Internet, color-coded by status
- Diagnosis card: pinpoints the exact failure (WiFi / ISP / DNS) in plain English
- Auto-run on a schedule: every 30 s · 1 min · 5 min · 10 min

---

### ⚡ Speed Test — *Download, upload, and latency*

Tests your connection speed via Cloudflare's public endpoints — no third-party account needed.

![Speed Test](docs/screenshots/speedtest.png)

- Semicircle gauges for latency, download, and upload
- Live area chart showing throughput over time
- Progressive file sizes for accurate measurement at any speed tier

---

### 🔎 DNS Lookup — *Query any record type from any resolver*

Inspect DNS records with full control over record type and resolver.

![DNS Lookup](docs/screenshots/dns.png)

- Record types: A · AAAA · MX · NS · TXT · CNAME · SOA · PTR · SRV
- Resolver picker: System · Google (8.8.8.8) · Cloudflare (1.1.1.1) · Quad9 · OpenDNS · Custom
- "All Types" button queries all common types in parallel

---

### 📋 Whois — *Domain registration and IP block ownership*

Look up domain registration details or IP block ownership instantly.

![Whois](docs/screenshots/whois.png)

- Highlight cards for: Domain · Registrar · Registered · Expires (turns red if < 60 days) · Organization · Country
- Full parsed sections (Registrant · Admin · Tech · Network)
- Raw output toggle

---

### 🔌 Port Scanner — *Find open services on any host*

Probe TCP ports to discover what services are running.

![Port Scanner](docs/screenshots/ports.png)

- Preset profiles: Common · Web · Database · Remote · Mail · Custom ranges
- Visual bubble grid — open ports glow green, closed ports dim
- Service descriptions for 30+ well-known ports

---

### 📶 MTR — *Continuous per-hop latency monitoring*

MTR (My Traceroute) combines traceroute and ping — it continuously probes every hop and tracks statistics over time.

![MTR](docs/screenshots/mtr.png)

- Per-hop stats: Loss% · Sent · Last · Avg · Best · Worst · StDev
- Live 30-point sparkline per hop, color-coded by RTT
- Stops / resets on demand

---

### 🔍 Network Scan — *Discover devices on your local network*

Scans the ARP table to find every device on your local network — no root or admin privileges required.

![Network Scan](docs/screenshots/netscan.png)

- Device cards: IP · MAC · vendor · hostname · device type (auto-detected from OUI prefix)
- **Grid view** and **Topology view** — star diagram with gateway at center
- Identifies routers, phones, TVs, printers, IoT devices, and more

---

### 🔒 SSL Inspector — *Check certificate validity and configuration*

Inspect the TLS certificate of any HTTPS host without leaving the app.

![SSL Inspector](docs/screenshots/ssl.png)

- Expiry countdown with color-coded banner (green / amber / red)
- Subject · Issuer · SANs · Protocol · Cipher suite · Serial · SHA-256 fingerprint
- Alerts when a certificate expires within 30 days

---

### 🌐 HTTP Headers — *Response headers, redirects, and security score*

Fetch the full HTTP response headers for any URL and get an instant security assessment.

![HTTP Headers](docs/screenshots/http.png)

- Complete header table with security headers highlighted green, info-leaking headers flagged amber
- Redirect chain — every hop with status codes
- Security score (0–100) with itemized findings (missing HSTS, CSP, X-Frame-Options, etc.)
- Response timing

---

### ⚡ Wake on LAN — *Wake up devices remotely*

Send a magic packet to any device on your local network to wake it from sleep.

![Wake on LAN](docs/screenshots/wol.png)

- Enter a MAC address and press Wake — it's instant
- Save frequently used devices (persisted locally)
- Accepts both `AA:BB:CC` and `AA-BB-CC` formats

---

## Download

Pre-built installers are available on the [Releases](https://github.com/krishghata/TraceAtlas/releases/latest) page:

| Platform | Format |
|----------|--------|
| Windows  | `.msi` installer · Portable `.zip` |
| macOS    | `.dmg` |
| Linux    | `.AppImage` · `.deb` |

The app auto-updates in the background — you'll see a banner when a new version is ready.

---

## Build from Source

See [docs/SETUP.md](docs/SETUP.md) for full prerequisites.

```bash
npm install
npm run tauri dev     # dev mode with hot-reload
npm run tauri build   # production build (~10–15 MB installer)
```

---

## Limitations

- IP geolocation is approximate — city-level accuracy not guaranteed
- Some hops timeout or are hidden by firewalls (`* * *`)
- SSL Inspector and HTTP Headers require `openssl` and `curl` in PATH (both included by default on Windows 10+, macOS, and Linux)
- Wake on LAN only works on the local broadcast domain (same network segment)
- Traceroute reflects your path at a point in time — internet routing changes dynamically
- Not available on mobile (network commands require OS-level execution)

---

## Future Scope

- RIPE Atlas integration — traceroute from probes worldwide
- Bandwidth monitor — real-time per-interface RX/TX rates
- 3D globe visualization
- BGP dataset integration and ASN graph
- Route anomaly detection
- Windows code signing (Microsoft Trusted Signing — free for open source)

---

## License

MIT © [Krishnendu Ghata](https://github.com/krishghata)

---

<div align="center">
  <sub>TraceAtlas · Built with Tauri · Vue 3 · Rust · Leaflet · ip-api.com</sub>
</div>
