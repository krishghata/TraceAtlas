
<template>
<div style="flex:1;display:flex;flex-direction:column;overflow:hidden;padding:12px 14px;gap:10px;box-sizing:border-box;background:#0a0f1e;color:white;font-family:'Segoe UI',sans-serif">

  <!-- ── Header ──────────────────────────────────────────────────────────── -->
  <div style="flex-shrink:0">
    <h2 style="margin:0 0 2px;font-size:15px;font-weight:700;color:#e2e8f0">Network Scanner</h2>
    <p style="margin:0;font-size:12px;color:#475569">Discover devices on your local network via ARP table — no root required</p>
  </div>

  <!-- ── Controls ───────────────────────────────────────────────────────── -->
  <div style="display:flex;gap:8px;align-items:center;flex-shrink:0;flex-wrap:wrap">
    <button @click="scan" :disabled="scanning"
      style="padding:7px 18px;background:#1d4ed8;color:white;border:none;border-radius:6px;font-size:13px;font-weight:600;cursor:pointer">
      {{ scanning ? 'Scanning…' : '🔍 Scan Network' }}
    </button>

    <!-- View toggle -->
    <div style="display:flex;border:1px solid #1e3a5f;border-radius:6px;overflow:hidden;margin-left:auto">
      <button @click="view = 'grid'" :style="`padding:5px 14px;border:none;font-size:11px;font-weight:600;cursor:pointer;
        background:${view==='grid'?'#1e3a5f':'transparent'};color:${view==='grid'?'#38bdf8':'#475569'}`">
        Grid
      </button>
      <button @click="view = 'topology'" :style="`padding:5px 14px;border:none;font-size:11px;font-weight:600;cursor:pointer;
        background:${view==='topology'?'#1e3a5f':'transparent'};color:${view==='topology'?'#38bdf8':'#475569'}`">
        Topology
      </button>
    </div>
  </div>

  <!-- ── Error ──────────────────────────────────────────────────────────── -->
  <div v-if="error" style="padding:8px 14px;background:#450a0a;color:#fca5a5;font-size:12px;border-radius:6px;border:1px solid #7f1d1d;flex-shrink:0">
    ⚠ {{ error }}
  </div>

  <!-- ── Scanning indicator ─────────────────────────────────────────────── -->
  <div v-if="scanning" style="flex-shrink:0">
    <div style="height:2px;background:#1e3a5f;border-radius:2px;overflow:hidden">
      <div style="height:100%;background:#38bdf8;border-radius:2px;animation:sweep 1.2s ease-in-out infinite"/>
    </div>
    <div style="margin-top:6px;font-size:11px;color:#475569">Reading ARP table and resolving hostnames…</div>
  </div>

  <!-- ── Empty state ────────────────────────────────────────────────────── -->
  <div v-if="!scanning && !devices.length && !error"
    style="flex:1;display:flex;align-items:center;justify-content:center;color:#334155;font-size:13px">
    Press Scan Network to discover local devices
  </div>

  <!-- ── Summary bar ────────────────────────────────────────────────────── -->
  <div v-if="devices.length" style="display:flex;gap:16px;align-items:center;flex-shrink:0;flex-wrap:wrap">
    <div style="font-size:12px;color:#64748b">
      <strong style="color:#e2e8f0">{{ devices.length }}</strong> devices found
    </div>
    <div style="font-size:12px;color:#64748b">
      Gateway: <strong style="color:#38bdf8;font-family:monospace">{{ gateway || '—' }}</strong>
    </div>
    <div style="font-size:12px;color:#64748b">
      Local IP: <strong style="color:#34d399;font-family:monospace">{{ localIp || '—' }}</strong>
    </div>
    <!-- Device type breakdown -->
    <div style="display:flex;gap:8px;margin-left:auto;flex-wrap:wrap">
      <span v-for="(count, type) in typeCounts" :key="type"
        style="padding:2px 10px;border-radius:10px;font-size:10px;font-weight:700;background:#0d1b2a;border:1px solid #1e3a5f;color:#94a3b8">
        {{ deviceIcon(type) }} {{ type }} ×{{ count }}
      </span>
    </div>
  </div>

  <!-- ── Grid view ──────────────────────────────────────────────────────── -->
  <div v-if="devices.length && view === 'grid'" style="flex:1;overflow-y:auto;min-height:0">
    <div style="display:grid;grid-template-columns:repeat(auto-fill,minmax(230px,1fr));gap:8px">
      <div v-for="dev in devices" :key="dev.ip"
        @click="selectedDevice = selectedDevice?.ip === dev.ip ? null : dev"
        :style="`padding:12px 14px;border-radius:8px;border:1px solid ${selectedDevice?.ip===dev.ip?'#38bdf8':'#1e3a5f'};
          background:${selectedDevice?.ip===dev.ip?'#0a1f3a':'#0d1b2a'};cursor:pointer;transition:all 0.15s`">

        <!-- Icon + type row -->
        <div style="display:flex;align-items:center;gap:8px;margin-bottom:8px">
          <span style="font-size:24px;line-height:1">{{ deviceIcon(dev.device_type) }}</span>
          <div>
            <div style="font-size:11px;font-weight:700;color:#94a3b8;text-transform:uppercase;letter-spacing:0.06em">
              {{ dev.device_type || 'Unknown' }}
            </div>
            <div style="font-size:10px;color:#475569">{{ dev.vendor || 'Unknown vendor' }}</div>
          </div>
        </div>

        <!-- IP -->
        <div style="font-size:14px;font-weight:700;color:#e2e8f0;font-family:monospace;margin-bottom:4px">
          {{ dev.ip }}
        </div>

        <!-- Hostname -->
        <div v-if="dev.hostname" style="font-size:11px;color:#64748b;font-family:monospace;margin-bottom:4px;word-break:break-all">
          {{ dev.hostname }}
        </div>

        <!-- MAC -->
        <div style="font-size:10px;color:#334155;font-family:monospace">{{ dev.mac }}</div>

        <!-- Gateway / Local badge -->
        <div style="display:flex;gap:6px;margin-top:8px;flex-wrap:wrap">
          <span v-if="dev.ip === gateway"
            style="padding:1px 8px;border-radius:10px;font-size:10px;font-weight:700;background:#1e3a5f;color:#38bdf8">
            Gateway
          </span>
          <span v-if="dev.ip === localIp"
            style="padding:1px 8px;border-radius:10px;font-size:10px;font-weight:700;background:#052e16;color:#22c55e">
            This Device
          </span>
        </div>
      </div>
    </div>
  </div>

  <!-- ── Topology view ──────────────────────────────────────────────────── -->
  <div v-if="devices.length && view === 'topology'" style="flex:1;overflow:hidden;min-height:0;display:flex;flex-direction:column">
    <svg ref="topoSvg" style="flex:1;width:100%;display:block" :viewBox="topoViewBox">

      <!-- Lines from gateway to each device -->
      <line v-for="dev in topoDevices" :key="'line-'+dev.ip"
        :x1="topoCenter.x" :y1="topoCenter.y"
        :x2="dev.tx" :y2="dev.ty"
        :stroke="dev.ip === localIp ? '#22c55e' : '#1e3a5f'"
        stroke-width="1"
        stroke-dasharray="4 4"/>

      <!-- Gateway node -->
      <g :transform="`translate(${topoCenter.x},${topoCenter.y})`">
        <circle r="28" fill="#0a1628" stroke="#38bdf8" stroke-width="2"/>
        <text text-anchor="middle" dominant-baseline="central" font-size="18">🌐</text>
        <text y="40" text-anchor="middle" fill="#38bdf8" font-size="9" font-family="monospace">{{ gateway }}</text>
        <text y="52" text-anchor="middle" fill="#475569" font-size="8">Gateway</text>
      </g>

      <!-- Device nodes -->
      <g v-for="dev in topoDevices" :key="'node-'+dev.ip"
        :transform="`translate(${dev.tx},${dev.ty})`"
        @click="selectedDevice = selectedDevice?.ip === dev.ip ? null : dev"
        style="cursor:pointer">
        <circle r="22"
          :fill="selectedDevice?.ip===dev.ip?'#0a1f3a':'#0d1b2a'"
          :stroke="dev.ip===localIp?'#22c55e':dev.ip===gateway?'#38bdf8':'#1e3a5f'"
          stroke-width="1.5"/>
        <text text-anchor="middle" dominant-baseline="central" font-size="14">{{ deviceIcon(dev.device_type) }}</text>
        <text y="32" text-anchor="middle" :fill="dev.ip===localIp?'#22c55e':'#94a3b8'" font-size="9" font-family="monospace">
          {{ dev.ip }}
        </text>
      </g>
    </svg>

    <!-- Selected device info below topology -->
    <div v-if="selectedDevice" style="flex-shrink:0;padding:10px 14px;background:#0d1b2a;border-radius:8px;border:1px solid #1e3a5f;margin-top:8px">
      <div style="display:flex;gap:14px;align-items:center;flex-wrap:wrap">
        <span style="font-size:22px">{{ deviceIcon(selectedDevice.device_type) }}</span>
        <div>
          <div style="font-size:14px;font-weight:700;color:#e2e8f0;font-family:monospace">{{ selectedDevice.ip }}</div>
          <div style="font-size:11px;color:#64748b">{{ selectedDevice.vendor || 'Unknown vendor' }} · {{ selectedDevice.device_type }}</div>
        </div>
        <div v-if="selectedDevice.hostname" style="font-size:12px;color:#94a3b8;font-family:monospace">{{ selectedDevice.hostname }}</div>
        <div style="font-size:11px;color:#334155;font-family:monospace">{{ selectedDevice.mac }}</div>
      </div>
    </div>
  </div>

  <!-- ── Device detail (grid view selected) ────────────────────────────── -->
  <div v-if="view === 'grid' && selectedDevice"
    style="flex-shrink:0;padding:12px 16px;border-radius:8px;border:1px solid #38bdf8;background:#0a1f3a">
    <div style="display:flex;gap:14px;align-items:flex-start;flex-wrap:wrap">
      <span style="font-size:28px;line-height:1">{{ deviceIcon(selectedDevice.device_type) }}</span>
      <div style="flex:1;min-width:180px">
        <div style="font-size:15px;font-weight:700;color:#e2e8f0;font-family:monospace">{{ selectedDevice.ip }}</div>
        <div style="font-size:12px;color:#64748b;margin-top:2px">{{ selectedDevice.vendor || 'Unknown vendor' }}</div>
      </div>
      <div v-if="selectedDevice.hostname" style="font-size:12px;color:#94a3b8;font-family:monospace;word-break:break-all">
        {{ selectedDevice.hostname }}
      </div>
      <div>
        <div style="font-size:10px;color:#475569;text-transform:uppercase;letter-spacing:0.07em;margin-bottom:2px">MAC Address</div>
        <div style="font-size:12px;color:#e2e8f0;font-family:monospace">{{ selectedDevice.mac }}</div>
      </div>
      <div>
        <div style="font-size:10px;color:#475569;text-transform:uppercase;letter-spacing:0.07em;margin-bottom:2px">Device Type</div>
        <div style="font-size:12px;color:#94a3b8">{{ selectedDevice.device_type || 'Unknown' }}</div>
      </div>
    </div>
  </div>

</div>
</template>

<script setup>
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const scanning       = ref(false)
const error          = ref(null)
const devices        = ref([])
const gateway        = ref('')
const localIp        = ref('')
const view           = ref('grid')   // 'grid' | 'topology'
const selectedDevice = ref(null)

// ── Device type → emoji ───────────────────────────────────────────────────────
function deviceIcon(type) {
  const map = {
    'Router':    '🌐',
    'Switch':    '🔀',
    'Phone':     '📱',
    'TV':        '📺',
    'Printer':   '🖨️',
    'Camera':    '📷',
    'IoT':       '💡',
    'Computer':  '💻',
    'Server':    '🖥️',
    'NAS':       '🗄️',
    'Unknown':   '❓',
  }
  return map[type] ?? '💻'
}

// ── Type count breakdown ──────────────────────────────────────────────────────
const typeCounts = computed(() => {
  const counts = {}
  for (const d of devices.value) {
    const t = d.device_type || 'Unknown'
    counts[t] = (counts[t] ?? 0) + 1
  }
  return counts
})

// ── Topology layout ───────────────────────────────────────────────────────────
const TOPO_W = 700
const TOPO_H = 460
const topoViewBox = `0 0 ${TOPO_W} ${TOPO_H}`
const topoCenter  = { x: TOPO_W / 2, y: TOPO_H / 2 }

const topoDevices = computed(() => {
  const devs = devices.value
  if (!devs.length) return []
  const count  = devs.length
  const radius = Math.min(Math.max(120, count * 22), 200)
  return devs.map((dev, i) => {
    const angle = (2 * Math.PI * i) / count - Math.PI / 2
    return {
      ...dev,
      tx: topoCenter.x + radius * Math.cos(angle),
      ty: topoCenter.y + radius * Math.sin(angle),
    }
  })
})

// ── Scan ──────────────────────────────────────────────────────────────────────
async function scan() {
  scanning.value       = true
  error.value          = null
  devices.value        = []
  selectedDevice.value = null

  try {
    const [devs, gw, ip] = await Promise.all([
      invoke('arp_scan'),
      invoke('get_default_gateway').catch(() => ''),
      invoke('get_local_ip').catch(() => ''),
    ])
    devices.value = devs
    gateway.value = typeof gw === 'string' ? gw.trim() : (gw?.gateway ?? '')
    localIp.value = typeof ip === 'string' ? ip.trim() : ''
  } catch (e) {
    error.value = String(e)
  } finally {
    scanning.value = false
  }
}
</script>

<style scoped>
@keyframes sweep {
  0%   { width: 15%; margin-left: 0; }
  50%  { width: 50%; margin-left: 30%; }
  100% { width: 15%; margin-left: 85%; }
}
</style>
