
<template>
<div style="flex:1;overflow:hidden;background:#0a0f1e">
<div style="flex:1;overflow-y:auto;min-height:0;height:100%;padding:32px 24px;box-sizing:border-box">

  <!-- ── Header ──────────────────────────────────────────────────────────── -->
  <div style="text-align:center;margin-bottom:36px">
    <img src="/logo-light.svg" alt="TraceAtlas" style="width:56px;height:56px;margin-bottom:12px"/>
    <h1 style="margin:0 0 6px;font-size:22px;font-weight:700;color:#e2e8f0">TraceAtlas</h1>
    <p style="margin:0;font-size:13px;color:#475569">Select a tool to open it in a new tab</p>
  </div>

  <!-- ── Feature grid ────────────────────────────────────────────────────── -->
  <div style="display:grid;grid-template-columns:repeat(auto-fill,minmax(200px,1fr));gap:12px;max-width:900px;margin:0 auto">
    <div v-for="f in features" :key="f.type"
      @click="$emit('open', f.type)"
      :style="`padding:20px 18px;border-radius:10px;border:1px solid ${f.open ? '#38bdf8' : '#1e3a5f'};
        background:${f.open ? '#0a1f3a' : '#0d1b2a'};cursor:pointer;transition:all 0.15s;
        display:flex;flex-direction:column;gap:10px;user-select:none`"
      @mouseenter="hover = f.type" @mouseleave="hover = null">

      <!-- Icon -->
      <div style="font-size:28px;line-height:1">{{ f.icon }}</div>

      <!-- Label + open badge -->
      <div style="display:flex;align-items:center;gap:8px;flex-wrap:wrap">
        <span style="font-size:13px;font-weight:700;color:#e2e8f0">{{ f.label }}</span>
        <span v-if="f.open"
          style="padding:1px 7px;border-radius:8px;font-size:10px;font-weight:700;background:#1e3a5f;color:#38bdf8">
          Open
        </span>
      </div>

      <!-- Description -->
      <p style="margin:0;font-size:11px;color:#475569;line-height:1.6">{{ f.desc }}</p>
    </div>
  </div>

  <!-- ── Footer ──────────────────────────────────────────────────────────── -->
  <div style="max-width:900px;margin:32px auto 0;display:flex;justify-content:space-between;align-items:center;padding-bottom:16px;flex-wrap:wrap;gap:8px">
    <span style="font-size:11px;color:#1e3a5f">TraceAtlas v0.3.0 · MIT License</span>
    <button @click="$emit('open', 'about')"
      style="background:none;border:none;font-size:11px;color:#334155;cursor:pointer;padding:4px 8px;border-radius:4px;transition:color 0.15s"
      @mouseenter="e => e.target.style.color='#64748b'"
      @mouseleave="e => e.target.style.color='#334155'">
      ℹ️ About &amp; Credits
    </button>
  </div>

</div>
</div>
</template>

<script setup>
import { ref } from 'vue'

const props = defineProps({ openTypes: Array })
defineEmits(['open'])

const hover = ref(null)

const FEATURES = [
  { type: 'trace',   icon: '🗺️',  label: 'Traceroute',      desc: 'Visualize your route hop by hop on a world map' },
  { type: 'ping',    icon: '📡',  label: 'Ping',             desc: 'Real-time latency chart with packet loss tracking' },
  { type: 'health',  icon: '🩺',  label: 'Network Health',   desc: 'Layer-by-layer diagnosis: local → router → ISP → DNS' },
  { type: 'speed',   icon: '⚡',  label: 'Speed Test',       desc: 'Measure download, upload speed and latency' },
  { type: 'dns',     icon: '🔎',  label: 'DNS Lookup',       desc: 'Query A, MX, TXT and more from any resolver' },
  { type: 'whois',   icon: '📋',  label: 'Whois',            desc: 'Domain registration info and IP block ownership' },
  { type: 'ports',   icon: '🔌',  label: 'Port Scanner',     desc: 'Probe TCP ports to discover open services' },
  { type: 'mtr',     icon: '📶',  label: 'MTR',              desc: 'Continuous per-hop latency stats — traceroute + ping' },
  { type: 'netscan', icon: '🔍',  label: 'Network Scan',     desc: 'Discover devices on your local network via ARP' },
  { type: 'ssl',     icon: '🔒',  label: 'SSL Inspector',    desc: 'Certificate validity, expiry, SANs and cipher suite' },
  { type: 'http',    icon: '🌐',  label: 'HTTP Headers',     desc: 'Response headers, redirects and security score' },
  { type: 'wol',     icon: '💡',  label: 'Wake on LAN',      desc: 'Send a magic packet to wake a sleeping device' },
]

const features = FEATURES.map(f => ({
  ...f,
  get open() { return props.openTypes?.includes(f.type) }
}))
</script>
