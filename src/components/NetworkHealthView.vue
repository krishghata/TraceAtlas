
<template>
<div style="flex:1;display:flex;flex-direction:column;overflow-y:auto;padding:16px;gap:14px;box-sizing:border-box;background:#0a0f1e;color:white;font-family:'Segoe UI',sans-serif">

  <!-- ── Header ──────────────────────────────────────────────────────────── -->
  <div style="display:flex;align-items:center;justify-content:space-between;flex-shrink:0">
    <div>
      <h2 style="margin:0;font-size:16px;font-weight:700;color:#e2e8f0">Network Health Check</h2>
      <p style="margin:4px 0 0;font-size:12px;color:#475569">Pings each layer independently — local network → ISP → internet → DNS</p>
    </div>
    <button @click="runCheck" :disabled="running"
      :style="`padding:8px 20px;border:none;border-radius:6px;font-size:13px;font-weight:600;cursor:${running?'default':'pointer'};color:white;background:${running?'#1e3a5f':'#1d4ed8'}`">
      {{ running ? 'Checking…' : layers.length ? '↺ Re-run' : '▶ Run Diagnosis' }}
    </button>
  </div>

  <!-- ── Empty state ─────────────────────────────────────────────────────── -->
  <div v-if="!layers.length"
    style="flex:1;display:flex;flex-direction:column;align-items:center;justify-content:center;gap:14px;color:#334155;text-align:center">
    <span style="font-size:56px">🩺</span>
    <div style="font-size:13px;color:#475569">Run a diagnosis to check your network layer by layer</div>
    <div style="font-size:11px;color:#1e3a5f;max-width:440px;line-height:1.7">
      Pings your router, ISP (via 8.8.8.8 and 1.1.1.1), and DNS separately to pinpoint whether the
      problem is in your home network, your ISP, or DNS.
    </div>
  </div>

  <template v-else>

    <!-- ═══════════════════════════════════════════════════════════════════ -->
    <!--  NETWORK PATH TOPOLOGY DIAGRAM                                      -->
    <!-- ═══════════════════════════════════════════════════════════════════ -->
    <div style="background:#0d1b2a;border-radius:10px;border:1px solid #1e3a5f;overflow:hidden;padding:6px 0 0;flex-shrink:0">
      <svg viewBox="0 0 760 136" style="width:100%;display:block">
        <defs>
          <filter id="glow-ok" x="-60%" y="-60%" width="220%" height="220%">
            <feGaussianBlur in="SourceGraphic" stdDeviation="3" result="b"/>
            <feMerge><feMergeNode in="b"/><feMergeNode in="SourceGraphic"/></feMerge>
          </filter>
          <filter id="glow-warn" x="-60%" y="-60%" width="220%" height="220%">
            <feGaussianBlur in="SourceGraphic" stdDeviation="2" result="b"/>
            <feMerge><feMergeNode in="b"/><feMergeNode in="SourceGraphic"/></feMerge>
          </filter>
          <!-- Per-node ring glow via radial gradient -->
          <radialGradient v-for="n in topoNodes" :key="'rg-'+n.id" :id="'rg-'+n.id"
            cx="50%" cy="50%" r="50%">
            <stop offset="60%" :stop-color="nodeRingColor(n.status)" stop-opacity="0"/>
            <stop offset="100%" :stop-color="nodeRingColor(n.status)" stop-opacity="0.35"/>
          </radialGradient>
        </defs>

        <!-- ── Connection lines ── -->
        <g v-for="line in topoLines" :key="line.key">
          <!-- Animated dashes while checking -->
          <line v-if="line.status === 'checking'"
            :x1="line.x1" :y1="64" :x2="line.x2" :y2="64"
            stroke="#1e3a5f" stroke-width="2.5" stroke-dasharray="7 5"
            class="flow"/>
          <!-- OK — solid green glow -->
          <line v-else-if="line.status === 'ok'"
            :x1="line.x1" :y1="64" :x2="line.x2" :y2="64"
            stroke="#22c55e" stroke-width="2.5" filter="url(#glow-ok)"/>
          <!-- Degraded — amber -->
          <line v-else-if="line.status === 'degraded'"
            :x1="line.x1" :y1="64" :x2="line.x2" :y2="64"
            stroke="#f59e0b" stroke-width="2.5" filter="url(#glow-warn)"/>
          <!-- Down — dashed red -->
          <line v-else
            :x1="line.x1" :y1="64" :x2="line.x2" :y2="64"
            stroke="#ef4444" stroke-width="2" stroke-dasharray="5 4"/>

          <!-- RTT label above mid-line -->
          <g v-if="line.rttLabel">
            <rect :x="(line.x1+line.x2)/2 - 18" y="38" width="36" height="16" rx="3"
              :fill="line.status === 'ok' ? '#052e16' : line.status === 'degraded' ? '#1a1000' : '#1a0000'"/>
            <text :x="(line.x1+line.x2)/2" y="50"
              text-anchor="middle" :fill="rttColor(line.rtt)"
              font-size="10" font-family="monospace" font-weight="bold">
              {{ line.rttLabel }}
            </text>
          </g>
          <!-- Checking label -->
          <text v-else-if="line.status === 'checking'"
            :x="(line.x1+line.x2)/2" y="50"
            text-anchor="middle" fill="#334155" font-size="9" font-family="monospace">…</text>
        </g>

        <!-- ── Nodes ── -->
        <g v-for="node in topoNodes" :key="node.id">
          <!-- Glow halo -->
          <circle v-if="node.status !== 'checking'"
            :cx="node.x" cy="64" r="36"
            :fill="`url(#rg-${node.id})`"/>
          <!-- Outer ring -->
          <circle :cx="node.x" cy="64" r="28"
            :stroke="nodeRingColor(node.status)" stroke-width="2.5" fill="none"
            :class="{'ring-pulse': node.status === 'checking'}"/>
          <!-- Inner background -->
          <circle :cx="node.x" cy="64" r="22" fill="#0a0f1e"/>
          <!-- Emoji icon -->
          <text :x="node.x" y="71" text-anchor="middle" font-size="18">{{ node.icon }}</text>
          <!-- Status dot top-right -->
          <circle v-if="node.status !== 'checking' && node.id !== 'you'"
            :cx="node.x + 19" :cy="64 - 19" r="5.5"
            :fill="nodeRingColor(node.status)"
            :style="node.status==='ok' ? 'filter:drop-shadow(0 0 4px #22c55e)' : ''"/>
          <!-- Node name -->
          <text :x="node.x" y="104"
            text-anchor="middle" fill="#e2e8f0" font-size="11" font-weight="bold"
            font-family="'Segoe UI',sans-serif">{{ node.label }}</text>
          <!-- Node sub-label -->
          <text :x="node.x" y="117"
            text-anchor="middle" fill="#475569" font-size="9.5" font-family="monospace">
            {{ node.sub }}
          </text>
        </g>
      </svg>
    </div>

    <!-- ═══════════════════════════════════════════════════════════════════ -->
    <!--  LAYER CARDS  (4-column)                                            -->
    <!-- ═══════════════════════════════════════════════════════════════════ -->
    <div style="display:grid;grid-template-columns:repeat(4,1fr);gap:10px;flex-shrink:0">
      <div v-for="layer in layers" :key="layer.id"
        :style="`padding:12px 14px;border-radius:8px;border:1px solid ${layerBorder(layer.status)};background:${layerBg(layer.status)}`">

        <!-- Card header -->
        <div style="display:flex;justify-content:space-between;align-items:flex-start;margin-bottom:8px">
          <div>
            <div style="font-size:12px;font-weight:600;color:#e2e8f0">{{ layer.name }}</div>
            <div style="font-size:10px;color:#475569;font-family:monospace;margin-top:2px">{{ layer.target }}
              <span v-if="layer.note" style="color:#334155;font-style:italic"> ({{ layer.note }})</span>
            </div>
          </div>
          <div :style="`padding:2px 8px;border-radius:12px;font-size:10px;font-weight:700;background:${badgeBg(layer.status)};color:${badgeColor(layer.status)}`">
            {{ statusLabel(layer.status) }}
          </div>
        </div>

        <!-- Metrics -->
        <div v-if="layer.status !== 'checking'" style="display:flex;gap:12px;margin-bottom:9px;font-size:11px">
          <span>
            <span style="color:#64748b">Avg </span>
            <span :style="`font-weight:700;font-family:monospace;color:${rttColor(layer.stats?.avg)}`">
              {{ layer.stats?.avg !== null ? layer.stats.avg + 'ms' : '—' }}
            </span>
          </span>
          <span>
            <span style="color:#64748b">Loss </span>
            <span :style="`font-weight:700;font-family:monospace;color:${lossCol(layer.stats?.loss)}`">
              {{ layer.stats?.loss !== undefined ? layer.stats.loss + '%' : '—' }}
            </span>
          </span>
        </div>
        <div v-else style="display:flex;align-items:center;gap:5px;margin-bottom:9px;font-size:11px;color:#334155">
          <span class="spinner"/>Checking…
        </div>

        <!-- Sparkline with area fill -->
        <div style="height:50px;border-radius:4px;overflow:hidden;background:rgba(0,0,0,0.25)">
          <svg viewBox="0 0 100 44" preserveAspectRatio="none" style="width:100%;height:100%;display:block">
            <defs>
              <linearGradient :id="'sg-'+layer.id" x1="0" y1="0" x2="0" y2="1">
                <stop offset="0%" :stop-color="layerLineColor(layer.status)" stop-opacity="0.45"/>
                <stop offset="100%" :stop-color="layerLineColor(layer.status)" stop-opacity="0"/>
              </linearGradient>
            </defs>
            <!-- Area fill -->
            <path v-if="sparkAreaPath(layer)" :d="sparkAreaPath(layer)"
              :fill="`url(#sg-${layer.id})`"/>
            <!-- Line -->
            <path v-if="sparkPath(layer)" :d="sparkPath(layer)"
              fill="none" :stroke="layerLineColor(layer.status)"
              stroke-width="1.8" stroke-linejoin="round"/>
            <!-- Dots -->
            <circle v-for="(sp, i) in sparkDots(layer)" :key="i"
              :cx="sp.x" :cy="sp.y" r="2.3" :fill="sp.color"/>
          </svg>
        </div>
      </div>
    </div>

    <!-- ═══════════════════════════════════════════════════════════════════ -->
    <!--  LATENCY COMPARISON BAR CHART                                       -->
    <!-- ═══════════════════════════════════════════════════════════════════ -->
    <div v-if="diagnosis" style="background:#0d1b2a;border-radius:8px;border:1px solid #1e3a5f;padding:14px;flex-shrink:0">
      <div style="font-size:11px;font-weight:600;color:#475569;margin-bottom:10px;letter-spacing:0.07em;text-transform:uppercase">
        Latency Comparison
      </div>
      <svg :viewBox="`0 0 500 ${layers.length * 26}`" style="width:100%;display:block">
        <defs>
          <linearGradient v-for="l in layers" :key="'bg-'+l.id" :id="'bg-'+l.id"
            x1="0" y1="0" x2="1" y2="0">
            <stop offset="0%" :stop-color="rttColor(l.stats?.avg)" stop-opacity="0.9"/>
            <stop offset="100%" :stop-color="rttColor(l.stats?.avg)" stop-opacity="0.4"/>
          </linearGradient>
        </defs>

        <g v-for="(layer, i) in layers" :key="layer.id">
          <!-- Row label -->
          <text x="0" :y="i * 26 + 15"
            fill="#64748b" font-size="10" font-family="'Segoe UI',sans-serif">
            {{ barLabel(layer) }}
          </text>
          <!-- Track -->
          <rect x="118" :y="i * 26 + 3" :width="BAR_MAX" height="16" rx="3" fill="#1e293b"/>
          <!-- Bar -->
          <rect v-if="layer.stats?.avg !== null && layer.status !== 'down'"
            x="118" :y="i * 26 + 3" :width="barW(layer.stats.avg)" height="16" rx="3"
            :fill="`url(#bg-${layer.id})`"
            :style="`filter:drop-shadow(0 0 4px ${rttColor(layer.stats.avg)}50)`"/>
          <!-- Down label -->
          <text v-else-if="layer.status === 'down'"
            x="124" :y="i * 26 + 15" fill="#ef4444" font-size="10" font-family="monospace">
            Unreachable
          </text>
          <!-- RTT value -->
          <text v-if="layer.stats?.avg !== null && layer.status !== 'down'"
            :x="118 + barW(layer.stats.avg) + 7" :y="i * 26 + 15"
            :fill="rttColor(layer.stats.avg)"
            font-size="10" font-family="monospace" font-weight="bold">
            {{ layer.stats.avg }}ms
          </text>
          <!-- Min / Max subtle annotation -->
          <text v-if="layer.stats?.min !== null && layer.status !== 'down'"
            x="468" :y="i * 26 + 15"
            fill="#334155" font-size="9" font-family="monospace" text-anchor="end">
            {{ layer.stats.min }}–{{ layer.stats.max }}
          </text>
        </g>
      </svg>
    </div>

    <!-- ═══════════════════════════════════════════════════════════════════ -->
    <!--  DIAGNOSIS CARD                                                      -->
    <!-- ═══════════════════════════════════════════════════════════════════ -->
    <div v-if="diagnosis"
      :style="`padding:16px 18px;border-radius:10px;border:1px solid ${diagBorder};background:${diagBg};flex-shrink:0`">
      <div style="display:flex;align-items:flex-start;gap:12px">
        <span style="font-size:28px;flex-shrink:0">{{ diagnosis.icon }}</span>
        <div style="flex:1">
          <div :style="`font-size:15px;font-weight:700;color:${diagColor};margin-bottom:4px`">{{ diagnosis.title }}</div>
          <div style="font-size:13px;color:#94a3b8;line-height:1.55;margin-bottom:6px">{{ diagnosis.message }}</div>
          <div v-if="diagnosis.suggestion"
            style="font-size:12px;color:#64748b;line-height:1.5;padding:8px 10px;background:rgba(0,0,0,0.3);border-radius:6px;border-left:3px solid #1e3a5f">
            💡 {{ diagnosis.suggestion }}
          </div>
        </div>
      </div>
    </div>

  </template>
</div>
</template>

<script setup>
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { parsePingOutput, rttColor } from '../lib/ping.js'

// ── Constants ────────────────────────────────────────────────────────────────
const BAR_MAX    = 340   // bar chart max width (SVG units)
const NODE_R     = 28    // topology node radius
const NODE_GAP   = 4     // gap between ring edge and line end

// ── State ────────────────────────────────────────────────────────────────────
const running   = ref(false)
const layers    = ref([])
const diagnosis = ref(null)

// ── Topology nodes & lines ───────────────────────────────────────────────────
const statusOrder = ['down', 'degraded', 'checking', 'ok']
function worstStatus(a, b) {
  return statusOrder.indexOf(a) < statusOrder.indexOf(b) ? a : b
}

const topoNodes = computed(() => {
  if (!layers.value.length) return []
  const [gw, isp1, isp2, dns] = layers.value

  // Collapse the two ISP pings into one node
  const ispStat = worstStatus(isp1.status ?? 'checking', isp2.status ?? 'checking')
  const ispAvgs = [isp1.stats?.avg, isp2.stats?.avg].filter(v => v != null)
  const ispAvg  = ispAvgs.length ? Math.round(ispAvgs.reduce((a, b) => a + b, 0) / ispAvgs.length) : null

  // Evenly space 4 nodes across 760px viewBox with 80px side margin
  return [
    { id: 'you',      x: 80,  icon: '🖥',  label: 'You',      sub: 'local device',       status: 'ok',     rtt: null  },
    { id: 'router',   x: 280, icon: '🏠',  label: 'Router',   sub: gw.target ?? '…',     status: gw.status  ?? 'checking', rtt: gw.stats?.avg  ?? null },
    { id: 'isp',      x: 480, icon: '🌐',  label: 'ISP',      sub: '8.8.8.8 · 1.1.1.1', status: ispStat,   rtt: ispAvg },
    { id: 'internet', x: 680, icon: '🌍',  label: 'Internet', sub: 'google.com',         status: dns.status ?? 'checking', rtt: dns.stats?.avg ?? null },
  ]
})

const topoLines = computed(() => {
  const nodes = topoNodes.value
  if (nodes.length < 2) return []
  return nodes.slice(0, -1).map((n, i) => {
    const next = nodes[i + 1]
    const rtt  = next.rtt
    return {
      key:      n.id + next.id,
      x1:       n.x    + NODE_R + NODE_GAP,
      x2:       next.x - NODE_R - NODE_GAP,
      status:   next.status,
      rtt,
      rttLabel: rtt !== null && next.status !== 'checking' ? rtt + 'ms' : null,
    }
  })
})

function nodeRingColor(status) {
  return { checking: '#1e3a5f', ok: '#22c55e', degraded: '#f59e0b', down: '#ef4444' }[status] ?? '#1e3a5f'
}

// ── Layer card helpers ────────────────────────────────────────────────────────
function layerBorder(s) { return { checking:'#1e3a5f', ok:'#166534', degraded:'#713f12', down:'#7f1d1d' }[s] ?? '#1e3a5f' }
function layerBg(s)     { return { checking:'#0a0f1e', ok:'#052e16', degraded:'#1a1000', down:'#1a0000' }[s] ?? '#0a0f1e' }
function badgeBg(s)     { return { checking:'#1e293b', ok:'#14532d', degraded:'#713f12', down:'#450a0a' }[s] ?? '#1e293b' }
function badgeColor(s)  { return { checking:'#64748b', ok:'#22c55e', degraded:'#f59e0b', down:'#ef4444' }[s] ?? '#64748b' }
function statusLabel(s) { return { checking:'Checking…', ok:'OK', degraded:'Degraded', down:'Down' }[s] ?? s }
function layerLineColor(s) { return { ok:'#22c55e', degraded:'#f59e0b', down:'#ef4444' }[s] ?? '#38bdf8' }
function lossCol(loss)  { return loss == null ? '#94a3b8' : loss === 0 ? '#22c55e' : loss < 20 ? '#f59e0b' : '#ef4444' }

// ── Sparkline helpers ────────────────────────────────────────────────────────
function sparkDots(layer) {
  const pkts = (layer.packets ?? []).slice(-8)
  if (!pkts.length) return []
  const rtts = pkts.filter(p => !p.dropped).map(p => p.rtt)
  const max  = rtts.length ? Math.max(50, Math.max(...rtts) * 1.1) : 100
  const step = pkts.length > 1 ? 100 / (pkts.length - 1) : 50
  return pkts.map((p, i) => ({
    x:     i * step,
    y:     p.dropped ? 40 : 38 - (p.rtt / max) * 32,
    color: rttColor(p.rtt),
  }))
}

function sparkPath(layer) {
  const pkts = (layer.packets ?? []).slice(-8)
  const dots = sparkDots(layer)
  let d = ''; let prev = null
  pkts.forEach((p, i) => {
    if (p.dropped) { prev = null; return }
    d += prev === null ? `M${dots[i].x},${dots[i].y} ` : `L${dots[i].x},${dots[i].y} `
    prev = dots[i]
  })
  return d || null
}

function sparkAreaPath(layer) {
  const pkts = (layer.packets ?? []).slice(-8)
  const dots = sparkDots(layer)
  const baseline = 42
  const runs = []; let cur = null
  pkts.forEach((p, i) => {
    if (p.dropped) { cur = null; return }
    if (!cur) { cur = []; runs.push(cur) }
    cur.push(dots[i])
  })
  let d = ''
  for (const run of runs) {
    if (run.length < 2) continue
    d += `M${run[0].x},${baseline} L${run[0].x},${run[0].y} `
    run.slice(1).forEach(pt => { d += `L${pt.x},${pt.y} ` })
    d += `L${run[run.length - 1].x},${baseline} Z `
  }
  return d || null
}

// ── Bar chart helpers ─────────────────────────────────────────────────────────
const maxBarRtt = computed(() => {
  const avgs = layers.value.map(l => l.stats?.avg).filter(v => v != null)
  return avgs.length ? Math.max(100, Math.max(...avgs) * 1.25) : 200
})

function barW(rtt) {
  return rtt == null ? 0 : Math.min((rtt / maxBarRtt.value) * BAR_MAX, BAR_MAX)
}

const BAR_LABELS = { gw: 'Local Gateway', isp1: 'ISP (8.8.8.8)', isp2: 'ISP (1.1.1.1)', dns: 'DNS (google)' }
function barLabel(layer) { return BAR_LABELS[layer.id] ?? layer.name }

// ── Diagnosis card ────────────────────────────────────────────────────────────
const diagBg     = computed(() => ({ ok:'#052e16', info:'#0a1628', warning:'#1a1000', critical:'#1a0000' })[diagnosis.value?.level] ?? '#0d1b2a')
const diagBorder = computed(() => ({ ok:'#166534', info:'#1e40af', warning:'#854d0e', critical:'#991b1b' })[diagnosis.value?.level] ?? '#1e3a5f')
const diagColor  = computed(() => ({ ok:'#22c55e', info:'#38bdf8', warning:'#f59e0b', critical:'#ef4444' })[diagnosis.value?.level] ?? '#e2e8f0')

function computeDiagnosis(ls) {
  const [gw, isp1, isp2, dns] = ls
  const gwDown     = gw.status   === 'down'
  const ispDown    = isp1.status === 'down'  && isp2.status === 'down'
  const ispDegrade = !ispDown && (isp1.status === 'degraded' || isp2.status === 'degraded')
  const dnsDown    = dns.status  === 'down'
  const dnsDegrade = dns.status  === 'degraded'
  const gwHighRtt  = gw.status   === 'ok' && (gw.stats?.avg ?? 0) > 20

  if (gwDown) return {
    level: 'critical', icon: '🔴',
    title: 'Local Network Issue',
    message: 'Cannot reach your router or gateway. Your device is not connected to the local network.',
    suggestion: 'Check your WiFi or Ethernet cable. Try restarting your router and wait 30 seconds.',
  }
  if (ispDown) return {
    level: 'critical', icon: '🔴',
    title: 'ISP Connection Down',
    message: 'Your router is reachable but internet is completely unreachable — the problem is between your router and your ISP.',
    suggestion: 'Try power-cycling your modem and router (off 30 seconds, on again). If it persists, contact your ISP — this is likely a service outage.',
  }
  if (ispDegrade) {
    const worstMs = Math.max(isp1.stats?.avg ?? 0, isp2.stats?.avg ?? 0)
    return {
      level: 'warning', icon: '🟡',
      title: 'ISP Connection Degraded',
      message: `Internet is reachable but with packet loss or high latency (avg ${worstMs}ms). This is typically an upstream ISP issue.`,
      suggestion: 'Try restarting your modem. If it persists over 30 minutes, contact your ISP to report degraded service.',
    }
  }
  if (dnsDown && !ispDown) return {
    level: 'warning', icon: '🟡',
    title: 'DNS Resolution Issue',
    message: 'Internet IPs (8.8.8.8, 1.1.1.1) are reachable but DNS hostname resolution is failing. Your DNS server may be down.',
    suggestion: 'Change your DNS to 8.8.8.8 (Google) or 1.1.1.1 (Cloudflare) in your network adapter settings, or flush your DNS cache.',
  }
  if (dnsDegrade && !ispDown) return {
    level: 'info', icon: '🟡',
    title: 'Minor Connectivity Issues',
    message: 'Network is mostly healthy but some packet loss detected to public hosts — possibly intermittent ISP congestion.',
    suggestion: 'Monitor for a few minutes. If packet loss persists above 5%, restart your router or contact your ISP.',
  }
  if (gwHighRtt) return {
    level: 'info', icon: '🟡',
    title: 'Possible WiFi Congestion',
    message: `Network is working but gateway latency is elevated (${gw.stats.avg}ms). This may indicate WiFi interference or a weak signal.`,
    suggestion: 'Try moving closer to your router, switching to 5 GHz, or using a wired Ethernet connection for better stability.',
  }
  const gwMs  = gw.stats?.avg   != null ? gw.stats.avg   + 'ms' : 'N/A'
  const ispMs = isp1.stats?.avg != null ? isp1.stats.avg + 'ms' : 'N/A'
  return {
    level: 'ok', icon: '🟢',
    title: 'Network Healthy',
    message: `All layers responding normally. Gateway: ${gwMs}, ISP: ${ispMs}, DNS working. No packet loss detected.`,
    suggestion: '',
  }
}

// ── Main check ────────────────────────────────────────────────────────────────
async function runCheck() {
  if (running.value) return
  running.value = true
  diagnosis.value = null

  let gwIp = '192.168.1.1', gwNote = 'Estimated'
  try {
    const res = await invoke('get_default_gateway')
    if (res.gateway) { gwIp = res.gateway; gwNote = 'Auto-detected' }
  } catch { /* ignore */ }

  layers.value = [
    { id: 'gw',   name: 'Local Gateway',   target: gwIp,         note: gwNote,       icon: '🏠', status: 'checking', stats: null, packets: [] },
    { id: 'isp1', name: 'ISP (Google)',     target: '8.8.8.8',    note: '',           icon: '🌐', status: 'checking', stats: null, packets: [] },
    { id: 'isp2', name: 'ISP (Cloudflare)', target: '1.1.1.1',    note: '',           icon: '☁️', status: 'checking', stats: null, packets: [] },
    { id: 'dns',  name: 'DNS Resolution',   target: 'google.com', note: 'needs DNS',  icon: '🔍', status: 'checking', stats: null, packets: [] },
  ]

  const pingLayer = async (idx) => {
    try {
      const res    = await invoke('run_ping', { target: layers.value[idx].target, count: 5 })
      const parsed = parsePingOutput(res.output, res.is_win)
      const s      = parsed.stats
      const status = s.loss === 100 ? 'down'
        : (s.loss > 20 || (s.avg != null && s.avg > 300)) ? 'degraded'
        : 'ok'
      layers.value[idx] = { ...layers.value[idx], stats: s, packets: parsed.packets, status }
    } catch {
      layers.value[idx] = { ...layers.value[idx], status: 'down',
        stats: { sent:0, received:0, loss:100, min:null, max:null, avg:null, jitter:null }, packets: [] }
    }
  }

  await Promise.allSettled([pingLayer(0), pingLayer(1), pingLayer(2), pingLayer(3)])
  diagnosis.value = computeDiagnosis(layers.value)
  running.value = false
}
</script>

<style scoped>
/* Topology line flow animation (checking state) */
@keyframes flow {
  from { stroke-dashoffset: 24; }
  to   { stroke-dashoffset: 0;  }
}
.flow { animation: flow 0.55s linear infinite; }

/* Node ring pulse (checking state) */
@keyframes ring-pulse {
  0%, 100% { opacity: 0.9; }
  50%       { opacity: 0.2; }
}
.ring-pulse { animation: ring-pulse 1.1s ease-in-out infinite; }

/* Card checking spinner */
.spinner {
  display: inline-block;
  width: 10px; height: 10px;
  border: 2px solid #1e3a5f;
  border-top-color: #38bdf8;
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
  flex-shrink: 0;
}
@keyframes spin { to { transform: rotate(360deg); } }
</style>
