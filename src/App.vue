
<template>
<div v-if="page === 'home'">
  <Landing @start="page = 'app'" />
</div>

<div v-else
  style="height:100vh;display:flex;flex-direction:column;overflow:hidden;background:#0a0f1e;color:white;font-family:'Segoe UI',sans-serif">

  <!-- ── Tab bar ─────────────────────────────────────────────────────────── -->
  <div style="display:flex;align-items:stretch;background:#0d1b2a;border-bottom:1px solid #1e3a5f;flex-shrink:0;padding:0 6px;overflow-x:auto;scrollbar-width:none">
    <button v-for="tab in tabs" :key="tab.id" @click="activeTab = tab.id"
      :style="`padding:8px 14px;background:none;border:none;border-bottom:2px solid ${activeTab===tab.id ? '#38bdf8' : 'transparent'};color:${activeTab===tab.id ? '#38bdf8' : '#64748b'};font-size:11px;font-weight:600;cursor:pointer;font-family:'Segoe UI',sans-serif;white-space:nowrap;transition:color 0.15s,border-color 0.15s`">
      {{ tab.label }}
    </button>
  </div>

  <!-- ── Update banner (shown when a newer version is available) ─────────── -->
  <div v-if="update && !updateDismissed"
    style="display:flex;align-items:center;gap:10px;padding:7px 14px;background:#052e16;border-bottom:1px solid #166534;flex-shrink:0;font-size:12px">
    <span style="color:#22c55e;font-size:14px">🆕</span>
    <span style="color:#94a3b8">
      <strong style="color:#e2e8f0">TraceAtlas {{ update.version }}</strong> is available
    </span>
    <template v-if="updateStatus === null">
      <button @click="installUpdate"
        style="padding:3px 12px;background:#166534;color:#22c55e;border:1px solid #22c55e;border-radius:4px;font-size:11px;font-weight:600;cursor:pointer">
        Install &amp; Restart
      </button>
      <button @click="updateDismissed = true"
        style="margin-left:auto;background:none;border:none;color:#475569;cursor:pointer;font-size:16px;padding:0 4px;line-height:1">
        ×
      </button>
    </template>
    <span v-else-if="updateStatus === 'downloading'" style="color:#64748b">
      ⏳ Downloading…
    </span>
    <span v-else-if="updateStatus === 'done'" style="color:#22c55e">
      ✓ Update installed — restart the app to apply it
    </span>
  </div>

  <!-- ── Active view fills remaining space ──────────────────────────────── -->
  <!-- KeepAlive preserves component state across tab switches -->
  <KeepAlive>
    <component :is="activeComponent" />
  </KeepAlive>

</div>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue'
import Landing           from './components/Landing.vue'
import AppView           from './components/AppView.vue'
import PingView          from './components/PingView.vue'
import NetworkHealthView from './components/NetworkHealthView.vue'
import SpeedTestView     from './components/SpeedTestView.vue'
import DnsView           from './components/DnsView.vue'
import WhoisView         from './components/WhoisView.vue'
import PortScannerView   from './components/PortScannerView.vue'
import MtrView           from './components/MtrView.vue'
import NetworkScanView   from './components/NetworkScanView.vue'
import SslView           from './components/SslView.vue'
import HttpInspectorView from './components/HttpInspectorView.vue'
import WakeOnLanView     from './components/WakeOnLanView.vue'
import { check } from '@tauri-apps/plugin-updater'

const page      = ref('home')
const activeTab = ref('trace')

const activeComponent = computed(() => ({
  trace:  AppView,
  ping:   PingView,
  health: NetworkHealthView,
  speed:  SpeedTestView,
  dns:     DnsView,
  whois:   WhoisView,
  ports:   PortScannerView,
  mtr:     MtrView,
  netscan: NetworkScanView,
  ssl:     SslView,
  http:    HttpInspectorView,
  wol:     WakeOnLanView,
})[activeTab.value])

const tabs = [
  { id: 'trace',  label: '⤷ Traceroute'     },
  { id: 'ping',   label: '📡 Ping'           },
  { id: 'health', label: '🩺 Network Health' },
  { id: 'speed',  label: '⚡ Speed Test'     },
  { id: 'dns',    label: '🔎 DNS Lookup'     },
  { id: 'whois',  label: '📋 Whois'          },
  { id: 'ports',   label: '🔌 Port Scanner'   },
  { id: 'mtr',     label: '📶 MTR'            },
  { id: 'netscan', label: '🔍 Network Scan'  },
  { id: 'ssl',     label: '🔒 SSL Inspector' },
  { id: 'http',    label: '🌐 HTTP Headers'  },
  { id: 'wol',     label: '⚡ Wake on LAN'   },
]

// ── Auto-updater ─────────────────────────────────────────────────────────────
const update          = ref(null)   // UpdateResult object when available
const updateStatus    = ref(null)   // null | 'downloading' | 'done'
const updateDismissed = ref(false)

async function checkForUpdates() {
  try {
    const result = await check()
    if (result?.available) update.value = result
  } catch {
    // Silently ignore — network may be offline or no release published yet
  }
}

async function installUpdate() {
  if (!update.value) return
  updateStatus.value = 'downloading'
  try {
    await update.value.downloadAndInstall()
    updateStatus.value = 'done'
  } catch {
    updateStatus.value = null   // reset so user can retry
  }
}

// Check 4 seconds after startup — non-blocking, does not affect launch speed
onMounted(() => setTimeout(checkForUpdates, 4000))
</script>
