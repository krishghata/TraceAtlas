
<template>
<div style="flex:1;overflow:hidden;background:#0a0f1e;color:white;font-family:'Segoe UI',sans-serif">
<div style="flex:1;overflow-y:auto;min-height:0;height:100%;padding:36px 28px;box-sizing:border-box">
<div style="max-width:680px;margin:0 auto;display:flex;flex-direction:column;gap:28px">

  <!-- ── App identity ────────────────────────────────────────────────── -->
  <div style="display:flex;align-items:center;gap:20px">
    <img src="/logo-light.svg" alt="TraceAtlas" style="width:64px;height:64px;flex-shrink:0;filter:drop-shadow(0 0 18px rgba(56,189,248,0.4))"/>
    <div>
      <h1 style="margin:0 0 4px;font-size:22px;font-weight:700;color:#e2e8f0">TraceAtlas</h1>
      <div style="display:flex;align-items:center;gap:10px;flex-wrap:wrap">
        <span style="padding:2px 10px;border-radius:10px;font-size:11px;font-weight:700;background:#1e3a5f;color:#38bdf8;font-family:monospace">
          v{{ VERSION }}
        </span>
        <span style="font-size:12px;color:#475569">Network Diagnostics Toolkit</span>
      </div>
      <p style="margin:8px 0 0;font-size:12px;color:#475569;line-height:1.7">
        A complete network diagnostics toolkit — 12 tools covering traceroute, ping,
        speed testing, DNS, port scanning, MTR, SSL inspection and more. Runs natively
        on Windows, macOS and Linux with no cloud dependency.
      </p>
      <a href="https://github.com/krishghata/TraceAtlas" target="_blank"
        style="display:inline-flex;align-items:center;gap:6px;margin-top:10px;font-size:12px;color:#38bdf8;text-decoration:none">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
          <path d="M12 0C5.37 0 0 5.37 0 12c0 5.31 3.435 9.795 8.205 11.385.6.105.825-.255.825-.57 0-.285-.015-1.23-.015-2.235-3.015.555-3.795-.735-4.035-1.41-.135-.345-.72-1.41-1.23-1.695-.42-.225-1.02-.78-.015-.795.945-.015 1.62.87 1.845 1.23 1.08 1.815 2.805 1.305 3.495.99.105-.78.42-1.305.765-1.605-2.67-.3-5.46-1.335-5.46-5.925 0-1.305.465-2.385 1.23-3.225-.12-.3-.54-1.53.12-3.18 0 0 1.005-.315 3.3 1.23.96-.27 1.98-.405 3-.405s2.04.135 3 .405c2.295-1.56 3.3-1.23 3.3-1.23.66 1.65.24 2.88.12 3.18.765.84 1.23 1.905 1.23 3.225 0 4.605-2.805 5.625-5.475 5.925.435.375.81 1.095.81 2.22 0 1.605-.015 2.895-.015 3.3 0 .315.225.69.825.57A12.02 12.02 0 0 0 24 12c0-6.63-5.37-12-12-12z"/>
        </svg>
        GitHub ↗
      </a>
    </div>
  </div>

  <!-- ── Divider ─────────────────────────────────────────────────────── -->
  <div style="height:1px;background:#1e3a5f"></div>

  <!-- ── Tools at a glance ───────────────────────────────────────────── -->
  <div>
    <h2 style="margin:0 0 14px;font-size:13px;font-weight:700;color:#64748b;text-transform:uppercase;letter-spacing:0.08em">Tools</h2>
    <div style="display:grid;grid-template-columns:repeat(auto-fill,minmax(200px,1fr));gap:6px">
      <div v-for="t in tools" :key="t.label"
        style="display:flex;align-items:center;gap:10px;padding:8px 12px;border-radius:7px;background:#0d1b2a;border:1px solid #1e3a5f">
        <span style="font-size:16px;line-height:1">{{ t.icon }}</span>
        <span style="font-size:12px;color:#94a3b8">{{ t.label }}</span>
      </div>
    </div>
  </div>

  <!-- ── Divider ─────────────────────────────────────────────────────── -->
  <div style="height:1px;background:#1e3a5f"></div>

  <!-- ── Built with ─────────────────────────────────────────────────── -->
  <div>
    <h2 style="margin:0 0 14px;font-size:13px;font-weight:700;color:#64748b;text-transform:uppercase;letter-spacing:0.08em">Built With</h2>
    <div style="display:flex;flex-wrap:wrap;gap:8px">
      <span v-for="s in stack" :key="s.name"
        :style="`padding:5px 14px;border-radius:8px;font-size:12px;font-weight:600;background:#0d1b2a;border:1px solid #1e3a5f;color:${s.color}`">
        {{ s.name }}
      </span>
    </div>
    <p style="margin:12px 0 0;font-size:11px;color:#334155;line-height:1.7">
      Desktop app built with Tauri v2 — Rust backend + native OS WebView. ~12 MB installer, ~40 MB RAM.
      No Chromium bundled. All network commands run on your machine.
    </p>
  </div>

  <!-- ── Divider ─────────────────────────────────────────────────────── -->
  <div style="height:1px;background:#1e3a5f"></div>

  <!-- ── Author ──────────────────────────────────────────────────────── -->
  <div>
    <h2 style="margin:0 0 14px;font-size:13px;font-weight:700;color:#64748b;text-transform:uppercase;letter-spacing:0.08em">Author</h2>
    <div style="display:flex;gap:18px;align-items:flex-start;flex-wrap:wrap">
      <!-- Avatar placeholder -->
      <div style="width:60px;height:60px;border-radius:50%;background:#0d1b2a;border:2px solid #1e3a5f;display:flex;align-items:center;justify-content:center;font-size:24px;flex-shrink:0">
        👤
      </div>
      <div style="flex:1;min-width:200px">
        <div style="font-size:15px;font-weight:700;color:#e2e8f0;margin-bottom:4px">Krishnendu Ghata</div>
        <div style="font-size:11px;color:#38bdf8;font-weight:600;margin-bottom:6px;letter-spacing:0.04em">
          AI Solution Architect &amp; Software Developer
        </div>
        <div style="font-size:12px;color:#475569;line-height:1.8">
          AI Solution Architect with a passion for developer tools, networking, and open-source software.
          TraceAtlas started as a personal tool for diagnosing network issues and grew into a
          full-featured diagnostics suite.
        </div>
        <div style="display:flex;gap:12px;margin-top:12px;flex-wrap:wrap">
          <a href="https://github.com/krishghata" target="_blank"
            style="display:flex;align-items:center;gap:6px;font-size:12px;color:#38bdf8;text-decoration:none">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
              <path d="M12 0C5.37 0 0 5.37 0 12c0 5.31 3.435 9.795 8.205 11.385.6.105.825-.255.825-.57 0-.285-.015-1.23-.015-2.235-3.015.555-3.795-.735-4.035-1.41-.135-.345-.72-1.41-1.23-1.695-.42-.225-1.02-.78-.015-.795.945-.015 1.62.87 1.845 1.23 1.08 1.815 2.805 1.305 3.495.99.105-.78.42-1.305.765-1.605-2.67-.3-5.46-1.335-5.46-5.925 0-1.305.465-2.385 1.23-3.225-.12-.3-.54-1.53.12-3.18 0 0 1.005-.315 3.3 1.23.96-.27 1.98-.405 3-.405s2.04.135 3 .405c2.295-1.56 3.3-1.23 3.3-1.23.66 1.65.24 2.88.12 3.18.765.84 1.23 1.905 1.23 3.225 0 4.605-2.805 5.625-5.475 5.925.435.375.81 1.095.81 2.22 0 1.605-.015 2.895-.015 3.3 0 .315.225.69.825.57A12.02 12.02 0 0 0 24 12c0-6.63-5.37-12-12-12z"/>
            </svg>
            krishghata
          </a>
          <a href="https://www.linkedin.com/in/krishnendu-ghata-45290050/" target="_blank"
            style="display:flex;align-items:center;gap:6px;font-size:12px;color:#38bdf8;text-decoration:none">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
              <path d="M20.447 20.452h-3.554v-5.569c0-1.328-.027-3.037-1.852-3.037-1.853 0-2.136 1.445-2.136 2.939v5.667H9.351V9h3.414v1.561h.046c.477-.9 1.637-1.85 3.37-1.85 3.601 0 4.267 2.37 4.267 5.455v6.286zM5.337 7.433a2.062 2.062 0 0 1-2.063-2.065 2.064 2.064 0 1 1 2.063 2.065zm1.782 13.019H3.555V9h3.564v11.452zM22.225 0H1.771C.792 0 0 .774 0 1.729v20.542C0 23.227.792 24 1.771 24h20.451C23.2 24 24 23.227 24 22.271V1.729C24 .774 23.2 0 22.222 0h.003z"/>
            </svg>
            LinkedIn
          </a>
        </div>
      </div>
    </div>
  </div>

  <!-- ── Divider ─────────────────────────────────────────────────────── -->
  <div style="height:1px;background:#1e3a5f"></div>

  <!-- ── License & links ─────────────────────────────────────────────── -->
  <div style="display:flex;justify-content:space-between;align-items:center;flex-wrap:wrap;gap:10px">
    <span style="font-size:12px;color:#334155">MIT License · Open Source</span>
    <a href="https://github.com/krishghata/TraceAtlas/releases" target="_blank"
      style="font-size:12px;color:#38bdf8;text-decoration:none">
      Changelog &amp; Releases ↗
    </a>
  </div>

</div>
</div>
</template>

<script setup>
const VERSION = '0.3.0'

const tools = [
  { icon: '🗺️', label: 'Traceroute'      },
  { icon: '📡', label: 'Ping'            },
  { icon: '🩺', label: 'Network Health'  },
  { icon: '⚡', label: 'Speed Test'      },
  { icon: '🔎', label: 'DNS Lookup'      },
  { icon: '📋', label: 'Whois'           },
  { icon: '🔌', label: 'Port Scanner'    },
  { icon: '📶', label: 'MTR'             },
  { icon: '🔍', label: 'Network Scan'    },
  { icon: '🔒', label: 'SSL Inspector'   },
  { icon: '🌐', label: 'HTTP Headers'    },
  { icon: '💡', label: 'Wake on LAN'     },
]

const stack = [
  { name: 'Tauri v2',   color: '#38bdf8' },
  { name: 'Vue 3',      color: '#4ade80' },
  { name: 'Rust',       color: '#fb923c' },
  { name: 'Leaflet',    color: '#a78bfa' },
  { name: 'Vite',       color: '#facc15' },
  { name: 'ip-api.com', color: '#94a3b8' },
]
</script>
