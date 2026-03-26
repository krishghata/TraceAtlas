
<template>
<!-- ── Splash ──────────────────────────────────────────────────────────────── -->
<div v-if="page === 'splash'">
  <Landing @start="page = 'app'" />
</div>

<!-- ── App shell ──────────────────────────────────────────────────────────── -->
<div v-else style="height:100vh;display:flex;flex-direction:column;overflow:hidden;background:#0a0f1e;color:white;font-family:'Segoe UI',sans-serif">

  <!-- ── Browser-style tab bar ─────────────────────────────────────────── -->
  <div style="display:flex;align-items:stretch;background:#060d18;border-bottom:1px solid #1e3a5f;flex-shrink:0;padding:0;overflow-x:auto;scrollbar-width:none;min-height:36px">

    <!-- Open tabs -->
    <div v-for="tab in openTabs" :key="tab.type"
      @click="activeType = tab.type; showHome = false"
      :style="`display:flex;align-items:center;gap:7px;padding:0 12px 0 14px;
        min-width:120px;max-width:180px;border-right:1px solid #1e3a5f;cursor:pointer;flex-shrink:0;
        background:${isActive(tab.type) ? '#0a0f1e' : 'transparent'};
        border-bottom:2px solid ${isActive(tab.type) ? '#38bdf8' : 'transparent'};
        transition:background 0.1s`">
      <span style="font-size:13px;line-height:1">{{ tab.icon }}</span>
      <span :style="`font-size:11px;font-weight:600;flex:1;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;
        color:${isActive(tab.type) ? '#e2e8f0' : '#64748b'}`">
        {{ tab.label }}
      </span>
      <!-- Close button -->
      <span @click.stop="closeTab(tab.type)"
        style="font-size:13px;color:#334155;cursor:pointer;padding:2px 2px;line-height:1;flex-shrink:0;border-radius:3px;transition:color 0.1s"
        @mouseenter="e => e.target.style.color='#94a3b8'"
        @mouseleave="e => e.target.style.color='#334155'">
        ×
      </span>
    </div>

    <!-- New tab (+) button -->
    <button @click="showHome = true; activeType = null"
      :style="`padding:0 14px;background:${showHome ? '#0a1628' : 'none'};border:none;
        border-right:1px solid #1e3a5f;border-bottom:2px solid ${showHome ? '#38bdf8' : 'transparent'};
        color:${showHome ? '#38bdf8' : '#475569'};font-size:18px;cursor:pointer;flex-shrink:0;
        transition:color 0.1s;line-height:1`"
      title="New tab">
      +
    </button>
  </div>

  <!-- ── Update banner ─────────────────────────────────────────────────── -->
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
    <span v-else-if="updateStatus === 'downloading'" style="color:#64748b">⏳ Downloading…</span>
    <span v-else-if="updateStatus === 'done'" style="color:#22c55e">✓ Update installed — restart to apply</span>
  </div>

  <!-- ── Home / feature picker ─────────────────────────────────────────── -->
  <HomeView v-if="showHome || !openTabs.length"
    :open-types="openTabs.map(t => t.type)"
    @open="openFeature"
    style="flex:1;overflow:hidden"/>

  <!-- ── Active tab content ─────────────────────────────────────────────── -->
  <!-- All open components are mounted and cached; only active one is visible -->
  <template v-if="openTabs.length">
    <KeepAlive>
      <component
        v-for="tab in openTabs"
        v-show="!showHome && isActive(tab.type)"
        :is="tab.component"
        :key="tab.type"
        style="flex:1;overflow:hidden;display:flex;flex-direction:column"/>
    </KeepAlive>
  </template>

</div>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue'
import Landing           from './components/Landing.vue'
import HomeView          from './components/HomeView.vue'
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
import AboutView        from './components/AboutView.vue'
import { check } from '@tauri-apps/plugin-updater'

// ── Feature registry ──────────────────────────────────────────────────────────
const FEATURES = {
  trace:   { type: 'trace',   icon: '🗺️',  label: 'Traceroute',    component: AppView           },
  ping:    { type: 'ping',    icon: '📡',  label: 'Ping',           component: PingView          },
  health:  { type: 'health',  icon: '🩺',  label: 'Network Health', component: NetworkHealthView },
  speed:   { type: 'speed',   icon: '⚡',  label: 'Speed Test',     component: SpeedTestView     },
  dns:     { type: 'dns',     icon: '🔎',  label: 'DNS Lookup',     component: DnsView           },
  whois:   { type: 'whois',   icon: '📋',  label: 'Whois',          component: WhoisView         },
  ports:   { type: 'ports',   icon: '🔌',  label: 'Port Scanner',   component: PortScannerView   },
  mtr:     { type: 'mtr',     icon: '📶',  label: 'MTR',            component: MtrView           },
  netscan: { type: 'netscan', icon: '🔍',  label: 'Network Scan',   component: NetworkScanView   },
  ssl:     { type: 'ssl',     icon: '🔒',  label: 'SSL Inspector',  component: SslView           },
  http:    { type: 'http',    icon: '🌐',  label: 'HTTP Headers',   component: HttpInspectorView },
  wol:     { type: 'wol',     icon: '💡',  label: 'Wake on LAN',    component: WakeOnLanView     },
  about:   { type: 'about',   icon: 'ℹ️',  label: 'About',          component: AboutView         },
}

// ── Tab state ─────────────────────────────────────────────────────────────────
const page      = ref('splash')
const openTabs  = ref([])       // array of feature objects (from FEATURES)
const activeType = ref(null)    // type string of the active tab
const showHome  = ref(false)    // true when home/picker is in focus

function isActive(type) {
  return !showHome.value && activeType.value === type
}

function openFeature(type) {
  const feature = FEATURES[type]
  if (!feature) return
  // If already open, just switch to it
  if (!openTabs.value.find(t => t.type === type)) {
    openTabs.value.push(feature)
  }
  activeType.value = type
  showHome.value   = false
}

function closeTab(type) {
  const idx = openTabs.value.findIndex(t => t.type === type)
  if (idx === -1) return
  openTabs.value.splice(idx, 1)

  if (activeType.value === type) {
    // Activate the adjacent tab, or show home if none left
    const next = openTabs.value[idx] ?? openTabs.value[idx - 1]
    if (next) {
      activeType.value = next.type
      showHome.value   = false
    } else {
      activeType.value = null
      showHome.value   = true
    }
  }
}

// ── Auto-updater ──────────────────────────────────────────────────────────────
const update          = ref(null)
const updateStatus    = ref(null)
const updateDismissed = ref(false)

async function checkForUpdates() {
  try {
    const result = await check()
    if (result?.available) update.value = result
  } catch { /* offline or no release yet */ }
}

async function installUpdate() {
  if (!update.value) return
  updateStatus.value = 'downloading'
  try {
    await update.value.downloadAndInstall()
    updateStatus.value = 'done'
  } catch {
    updateStatus.value = null
  }
}

onMounted(() => setTimeout(checkForUpdates, 4000))
</script>
