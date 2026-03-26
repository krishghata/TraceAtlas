
<template>
<div style="flex:1;display:flex;flex-direction:column;overflow:hidden;padding:10px 14px;gap:8px;box-sizing:border-box;background:#0a0f1e;color:white;font-family:'Segoe UI',sans-serif">

  <!-- ── Controls ────────────────────────────────────────────────────────── -->
  <div style="display:flex;gap:8px;align-items:center;flex-shrink:0">
    <input v-model="target" @keyup.enter="toggle"
      placeholder="Target — e.g. 8.8.8.8 or google.com"
      style="flex:1;padding:7px 12px;background:#0d1b2a;border:1px solid #1e3a5f;border-radius:6px;color:white;font-size:13px;outline:none"/>

    <select v-model="mode" :disabled="running"
      style="padding:7px 10px;background:#0d1b2a;border:1px solid #1e3a5f;border-radius:6px;color:#94a3b8;font-size:12px;cursor:pointer">
      <option value="continuous">Continuous</option>
      <option value="10">10 packets</option>
      <option value="20">20 packets</option>
      <option value="50">50 packets</option>
    </select>

    <button @click="toggle"
      :style="`padding:7px 18px;border:none;border-radius:6px;font-size:13px;font-weight:600;cursor:pointer;color:white;background:${running ? '#7f1d1d' : '#1d4ed8'}`">
      {{ running ? '⏹ Stop' : '▶ Start' }}
    </button>

    <button @click="clearData" :disabled="running || !packets.length"
      style="padding:7px 12px;background:#1e293b;color:#94a3b8;border:1px solid #334155;border-radius:6px;font-size:12px;cursor:pointer">
      Clear
    </button>
  </div>

  <!-- ── Stats bar ───────────────────────────────────────────────────────── -->
  <div style="display:flex;gap:0;padding:10px 14px;background:#0d1b2a;border-radius:8px;border:1px solid #1e3a5f;flex-shrink:0;flex-wrap:wrap;align-items:center">
    <div v-for="s in statItems" :key="s.label" style="display:flex;flex-direction:column;align-items:center;padding:0 14px;border-right:1px solid #1e3a5f">
      <span style="font-size:11px;color:#475569;margin-bottom:2px">{{ s.label }}</span>
      <span :style="`font-size:16px;font-weight:700;font-family:monospace;color:${s.color}`">{{ s.value }}</span>
    </div>
    <div style="margin-left:auto;display:flex;align-items:center;gap:6px;padding-left:14px">
      <span :style="`width:8px;height:8px;border-radius:50%;flex-shrink:0;background:${running ? '#22c55e' : '#334155'};${running ? 'box-shadow:0 0 8px #22c55e;animation:pulse 1s infinite' : ''}`"></span>
      <span style="font-size:11px;color:#64748b">{{ running ? 'Pinging…' : packets.length ? 'Stopped' : 'Idle' }}</span>
    </div>
  </div>

  <!-- ── Charts row — fixed height ─────────────────────────────────────── -->
  <div style="height:220px;flex-shrink:0;display:flex;gap:10px">

    <!-- Donut chart — 30% -->
    <div style="flex:3;background:#0d1b2a;border-radius:8px;border:1px solid #1e3a5f;display:flex;flex-direction:column;align-items:center;justify-content:center;gap:8px;padding:12px">
      <div style="font-size:10px;text-transform:uppercase;letter-spacing:0.07em;color:#475569">Success / Failure</div>
      <svg viewBox="0 0 100 100" style="width:100px;height:100px">
        <circle cx="50" cy="50" r="36" fill="none" stroke="#0a1628" stroke-width="12"/>
        <circle cx="50" cy="50" r="36" fill="none" stroke="#7f1d1d" stroke-width="12"
          stroke-dasharray="226 226" transform="rotate(-90 50 50)"/>
        <circle cx="50" cy="50" r="36" fill="none" :stroke="donut.successColor" stroke-width="12"
          :stroke-dasharray="`${donut.successLen} 226`" stroke-linecap="round" transform="rotate(-90 50 50)"/>
        <text x="50" y="46" text-anchor="middle" font-size="16" font-weight="700" :fill="donut.successColor" font-family="monospace">{{ donut.pct }}%</text>
        <text x="50" y="60" text-anchor="middle" font-size="8" fill="#475569" font-family="sans-serif">success</text>
      </svg>
      <div style="display:flex;gap:14px;font-size:11px">
        <span style="display:flex;align-items:center;gap:4px">
          <span style="width:7px;height:7px;border-radius:50%;background:#22c55e;display:inline-block"></span>
          <span style="color:#64748b">{{ stats.received }} ok</span>
        </span>
        <span style="display:flex;align-items:center;gap:4px">
          <span style="width:7px;height:7px;border-radius:50%;background:#7f1d1d;display:inline-block"></span>
          <span style="color:#64748b">{{ stats.sent - stats.received }} lost</span>
        </span>
      </div>
    </div>

    <!-- Line chart — 40% -->
    <div style="flex:4;background:#0d1b2a;border-radius:8px;border:1px solid #1e3a5f;overflow:hidden;position:relative">
      <div v-if="!packets.length" style="position:absolute;inset:0;display:flex;align-items:center;justify-content:center;color:#334155;font-size:13px">
        Press Start to see the live latency chart
      </div>
      <svg v-else :viewBox="`0 0 ${SVG_W} ${SVG_H}`" preserveAspectRatio="none"
        style="width:100%;height:100%;display:block">
        <g v-for="g in gridLines" :key="g.label">
          <line :x1="PL" :y1="g.y" :x2="SVG_W - PR" :y2="g.y" stroke="#1e3a5f" stroke-width="0.8"/>
          <text :x="PL - 5" :y="g.y + 3.5" text-anchor="end" fill="#334155" font-size="9" font-family="monospace">{{ g.label }}</text>
        </g>
        <text v-for="xl in xLabels" :key="xl.seq" :x="xl.cx" :y="SVG_H - 2"
          text-anchor="middle" fill="#334155" font-size="9" font-family="monospace">{{ xl.seq }}</text>
        <g v-for="pt in droppedPoints" :key="'x' + pt.seq">
          <line :x1="pt.cx - 4" :y1="SVG_H - PB - 10" :x2="pt.cx + 4" :y2="SVG_H - PB" stroke="#ef4444" stroke-width="1.5"/>
          <line :x1="pt.cx + 4" :y1="SVG_H - PB - 10" :x2="pt.cx - 4" :y2="SVG_H - PB" stroke="#ef4444" stroke-width="1.5"/>
        </g>
        <path v-if="areaPath" :d="areaPath" fill="rgba(56,189,248,0.06)"/>
        <path v-if="linePath" :d="linePath" fill="none" stroke="#38bdf8" stroke-width="1.5" stroke-linejoin="round"/>
        <circle v-for="pt in validPoints" :key="'dot' + pt.seq"
          :cx="pt.cx" :cy="pt.cy" r="2.8" :fill="pt.color" :style="`filter:drop-shadow(0 0 3px ${pt.color})`"/>
      </svg>
    </div>

    <!-- Recent pings feed — 30% -->
    <div style="flex:3;background:#0d1b2a;border-radius:8px;border:1px solid #1e3a5f;overflow:hidden;display:flex;flex-direction:column">
      <div style="padding:6px 12px;background:#0a1628;border-bottom:1px solid #1e3a5f;font-size:10px;font-weight:600;color:#475569;text-transform:uppercase;letter-spacing:0.07em;flex-shrink:0">
        Recent Pings
      </div>
      <div v-if="!packets.length" style="flex:1;display:flex;align-items:center;justify-content:center;color:#334155;font-size:12px">
        —
      </div>
      <div v-else style="flex:1;overflow-y:auto;padding:6px 8px;display:flex;flex-direction:column;gap:3px">
        <div v-for="p in recentPings" :key="p.seq"
          style="display:flex;align-items:center;gap:6px;padding:3px 6px;border-radius:4px;background:#0a0f1e">
          <span style="font-size:10px;color:#334155;font-family:monospace;width:28px;flex-shrink:0">#{{ p.seq }}</span>
          <div style="flex:1;height:4px;border-radius:2px;background:#0d1b2a;overflow:hidden">
            <div :style="`height:100%;border-radius:2px;width:${p.barW}%;background:${p.color};transition:width 0.2s`"></div>
          </div>
          <span :style="`font-size:11px;font-family:monospace;width:46px;text-align:right;flex-shrink:0;color:${p.color}`">
            {{ p.dropped ? 'timeout' : p.rtt + ' ms' }}
          </span>
        </div>
      </div>
    </div>

  </div>

</div>
</template>

<script setup>
import { ref, computed, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { parsePingOutput, rttColor } from '../lib/ping.js'

// ── SVG constants ───────────────────────────────────────────────────────────
const SVG_W    = 600
const SVG_H    = 200
const PL       = 36   // padding left  (Y-axis labels)
const PR       = 6    // padding right
const PT       = 8    // padding top
const PB       = 16   // padding bottom (X-axis labels)
const CW       = SVG_W - PL - PR   // chart width
const CH       = SVG_H - PT - PB   // chart height
const MAX_VIS  = 60                 // max visible packets in chart
const PS       = CW / MAX_VIS       // pixels per packet slot

// ── State ───────────────────────────────────────────────────────────────────
const target  = ref('8.8.8.8')
const mode    = ref('continuous')
const running = ref(false)
const packets = ref([])             // { rtt, dropped, seq }
let   seq     = 0

// ── Stats ───────────────────────────────────────────────────────────────────
const stats = computed(() => {
  const all  = packets.value
  if (!all.length) return { sent: 0, received: 0, loss: 0, min: null, max: null, avg: null, jitter: null }
  const rtts    = all.filter(p => !p.dropped).map(p => p.rtt)
  const dropped = all.filter(p => p.dropped).length
  const jitter  = rtts.length > 1
    ? Math.round(rtts.slice(1).reduce((s, r, i) => s + Math.abs(r - rtts[i]), 0) / (rtts.length - 1))
    : null
  return {
    sent:     all.length,
    received: rtts.length,
    loss:     Math.round((dropped / all.length) * 100),
    min:      rtts.length ? Math.min(...rtts) : null,
    max:      rtts.length ? Math.max(...rtts) : null,
    avg:      rtts.length ? Math.round(rtts.reduce((a, b) => a + b, 0) / rtts.length) : null,
    jitter,
  }
})

// ── Recent pings feed ────────────────────────────────────────────────────────
const recentPings = computed(() => {
  const last = packets.value.slice(-20).reverse()
  const maxRttFeed = Math.max(1, ...last.filter(p => !p.dropped).map(p => p.rtt))
  return last.map(p => ({
    ...p,
    color:  p.dropped ? '#ef4444' : rttColor(p.rtt),
    barW:   p.dropped ? 100 : Math.round((p.rtt / maxRttFeed) * 100),
  }))
})

// ── Donut chart ─────────────────────────────────────────────────────────────
const CIRC = 2 * Math.PI * 36  // ≈ 226
const donut = computed(() => {
  const s = stats.value
  if (!s.sent) return { pct: 0, successLen: 0, successColor: '#334155' }
  const pct = Math.round((s.received / s.sent) * 100)
  const color = pct === 100 ? '#22c55e' : pct >= 90 ? '#f59e0b' : '#ef4444'
  return { pct, successLen: (s.received / s.sent) * CIRC, successColor: color }
})

const lossColor = computed(() => {
  const l = stats.value.loss
  return l === 0 ? '#22c55e' : l < 10 ? '#f59e0b' : '#ef4444'
})

const statItems = computed(() => [
  { label: 'Sent',     value: stats.value.sent     || '—',                         color: '#94a3b8' },
  { label: 'Received', value: stats.value.received || '—',                         color: '#94a3b8' },
  { label: 'Loss',     value: stats.value.sent ? stats.value.loss + '%' : '—',     color: lossColor.value },
  { label: 'Min',      value: stats.value.min  !== null ? stats.value.min  + ' ms' : '—', color: rttColor(stats.value.min) },
  { label: 'Avg',      value: stats.value.avg  !== null ? stats.value.avg  + ' ms' : '—', color: rttColor(stats.value.avg) },
  { label: 'Max',      value: stats.value.max  !== null ? stats.value.max  + ' ms' : '—', color: rttColor(stats.value.max) },
  { label: 'Jitter',   value: stats.value.jitter !== null ? stats.value.jitter + ' ms' : '—', color: '#94a3b8' },
])

// ── Chart computeds ─────────────────────────────────────────────────────────
const visible = computed(() => packets.value.slice(-MAX_VIS))

const maxRtt = computed(() => {
  const rtts = visible.value.filter(p => !p.dropped).map(p => p.rtt)
  return rtts.length ? Math.max(100, Math.ceil(Math.max(...rtts) * 1.15)) : 200
})

function ptCoords(p, i) {
  const cx = PL + (i + 0.5) * PS
  const cy = p.dropped ? SVG_H - PB : SVG_H - PB - (p.rtt / maxRtt.value) * CH
  return { ...p, cx, cy, color: rttColor(p.rtt) }
}

const chartPoints  = computed(() => visible.value.map((p, i) => ptCoords(p, i)))
const validPoints  = computed(() => chartPoints.value.filter(p => !p.dropped))
const droppedPoints = computed(() => chartPoints.value.filter(p => p.dropped))

/** SVG <path d="…"> for the RTT line — MoveTo at each run start so gaps appear at dropouts. */
const linePath = computed(() => {
  let d = ''; let prev = null
  for (const pt of chartPoints.value) {
    if (pt.dropped) { prev = null; continue }
    d += prev === null ? `M${pt.cx},${pt.cy} ` : `L${pt.cx},${pt.cy} `
    prev = pt
  }
  return d || null
})

/** Filled area under the line for visual depth. */
const areaPath = computed(() => {
  const runs = []  // each run: array of pts
  let cur = null
  for (const pt of chartPoints.value) {
    if (pt.dropped) { cur = null; continue }
    if (!cur) { cur = []; runs.push(cur) }
    cur.push(pt)
  }
  let d = ''
  const base = SVG_H - PB
  for (const run of runs) {
    if (run.length < 2) continue
    d += `M${run[0].cx},${base} L${run[0].cx},${run[0].cy} `
    run.slice(1).forEach(p => { d += `L${p.cx},${p.cy} ` })
    d += `L${run[run.length - 1].cx},${base} Z `
  }
  return d || null
})

const gridLines = computed(() => {
  const max = maxRtt.value
  return [0.25, 0.5, 0.75, 1].map(f => ({
    y:     SVG_H - PB - f * CH,
    label: Math.round(max * f) + 'ms',
  }))
})

const xLabels = computed(() => {
  const vis = visible.value
  const base = packets.value.length - vis.length  // seq offset
  return vis
    .map((_, i) => ({ seq: base + i + 1, cx: PL + (i + 0.5) * PS }))
    .filter((_, i) => i % 10 === 0)
})

// ── Ping logic ───────────────────────────────────────────────────────────────
async function pingOnce() {
  const t = target.value.trim()
  try {
    const res    = await invoke('run_ping', { target: t, count: 1 })
    const parsed = parsePingOutput(res.output, res.is_win)
    for (const p of parsed.packets) packets.value.push({ ...p, seq: ++seq })
    // If parsed returned nothing (unexpected output) treat as drop
    if (!parsed.packets.length) packets.value.push({ rtt: null, dropped: true, seq: ++seq })
  } catch {
    packets.value.push({ rtt: null, dropped: true, seq: ++seq })
  }
}

async function pingBatch(count) {
  const t = target.value.trim()
  try {
    const res    = await invoke('run_ping', { target: t, count })
    const parsed = parsePingOutput(res.output, res.is_win)
    for (const p of parsed.packets) packets.value.push({ ...p, seq: ++seq })
  } catch {
    for (let i = 0; i < count; i++) packets.value.push({ rtt: null, dropped: true, seq: ++seq })
  }
}

async function toggle() {
  if (running.value) { running.value = false; return }
  const t = target.value.trim()
  if (!t) return

  running.value = true

  if (mode.value === 'continuous') {
    while (running.value) {
      await pingOnce()
      if (running.value) await new Promise(r => setTimeout(r, 250))
    }
  } else {
    await pingBatch(parseInt(mode.value, 10))
    running.value = false
  }
}

function clearData() {
  if (running.value) return
  packets.value = []
  seq = 0
}

onUnmounted(() => { running.value = false })
</script>
