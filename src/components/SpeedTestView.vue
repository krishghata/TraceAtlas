
<template>
<div style="flex:1;display:flex;flex-direction:column;overflow:hidden;background:#0a0f1e;color:white;font-family:'Segoe UI',sans-serif">
<div style="flex:1;overflow-y:auto;min-height:0;padding:14px;gap:12px;display:flex;flex-direction:column;box-sizing:border-box">

  <!-- ── Header ──────────────────────────────────────────────────────────── -->
  <div style="display:flex;align-items:center;justify-content:space-between;flex-shrink:0">
    <div>
      <h2 style="margin:0;font-size:16px;font-weight:700;color:#e2e8f0">Speed Test</h2>
      <p style="margin:4px 0 0;font-size:12px;color:#475569">Measures download speed, upload speed, and latency via Cloudflare</p>
    </div>
    <button @click="startTest" :disabled="phase !== 'idle' && phase !== 'done'"
      :style="`padding:8px 20px;border:none;border-radius:6px;font-size:13px;font-weight:600;cursor:${phase==='idle'||phase==='done'?'pointer':'default'};color:white;background:${phase==='idle'||phase==='done'?'#1d4ed8':'#1e3a5f'}`">
      {{ phase === 'idle' ? '▶ Start Test' : phase === 'done' ? '↺ Re-test' : 'Testing…' }}
    </button>
  </div>

  <!-- ── Gauge row ───────────────────────────────────────────────────────── -->
  <div style="display:grid;grid-template-columns:1fr 1fr 1fr;gap:10px;flex-shrink:0">

    <!-- Ping gauge -->
    <div style="background:#0d1b2a;border-radius:10px;border:1px solid #1e3a5f;padding:16px;display:flex;flex-direction:column;align-items:center;gap:6px">
      <div style="font-size:11px;font-weight:600;color:#475569;text-transform:uppercase;letter-spacing:0.07em">Latency</div>
      <svg viewBox="0 0 120 70" style="width:120px;height:70px">
        <path d="M 10 60 A 50 50 0 0 1 110 60" fill="none" stroke="#1e3a5f" stroke-width="8" stroke-linecap="round"/>
        <path v-if="results.ping !== null"
          d="M 10 60 A 50 50 0 0 1 110 60" fill="none"
          :stroke="pingColor" stroke-width="8" stroke-linecap="round"
          :stroke-dasharray="arcDasharray(results.ping, 300)"
          style="transition:stroke-dasharray 0.4s ease"/>
        <text x="60" y="56" text-anchor="middle" :fill="results.ping !== null ? pingColor : '#334155'"
          font-size="18" font-weight="700" font-family="monospace">
          {{ results.ping !== null ? results.ping : (phase === 'ping' ? '…' : '—') }}
        </text>
        <text x="60" y="68" text-anchor="middle" fill="#475569" font-size="8" font-family="monospace">ms</text>
      </svg>
      <div :style="`font-size:11px;color:${pingColor};font-weight:600`">
        {{ pingLabel }}
      </div>
    </div>

    <!-- Download gauge -->
    <div style="background:#0d1b2a;border-radius:10px;border:1px solid #1e3a5f;padding:16px;display:flex;flex-direction:column;align-items:center;gap:6px">
      <div style="font-size:11px;font-weight:600;color:#475569;text-transform:uppercase;letter-spacing:0.07em">Download</div>
      <svg viewBox="0 0 120 70" style="width:120px;height:70px">
        <path d="M 10 60 A 50 50 0 0 1 110 60" fill="none" stroke="#1e3a5f" stroke-width="8" stroke-linecap="round"/>
        <path v-if="liveDown > 0"
          d="M 10 60 A 50 50 0 0 1 110 60" fill="none" stroke="#38bdf8" stroke-width="8" stroke-linecap="round"
          :stroke-dasharray="downArcD"
          style="transition:stroke-dasharray 0.3s ease"/>
        <text x="60" y="56" text-anchor="middle" :fill="liveDown > 0 ? '#38bdf8' : '#334155'"
          font-size="18" font-weight="700" font-family="monospace">
          {{ liveDown > 0 ? liveDown : (phase === 'download' ? '…' : '—') }}
        </text>
        <text x="60" y="68" text-anchor="middle" fill="#475569" font-size="8" font-family="monospace">Mbps</text>
      </svg>
      <div style="font-size:11px;color:#38bdf8;font-weight:600">
        {{ results.download !== null ? results.download + ' Mbps peak' : (phase === 'download' ? 'Measuring…' : '') }}
      </div>
    </div>

    <!-- Upload gauge -->
    <div style="background:#0d1b2a;border-radius:10px;border:1px solid #1e3a5f;padding:16px;display:flex;flex-direction:column;align-items:center;gap:6px">
      <div style="font-size:11px;font-weight:600;color:#475569;text-transform:uppercase;letter-spacing:0.07em">Upload</div>
      <svg viewBox="0 0 120 70" style="width:120px;height:70px">
        <path d="M 10 60 A 50 50 0 0 1 110 60" fill="none" stroke="#1e3a5f" stroke-width="8" stroke-linecap="round"/>
        <path v-if="liveUp > 0"
          d="M 10 60 A 50 50 0 0 1 110 60" fill="none" stroke="#a78bfa" stroke-width="8" stroke-linecap="round"
          :stroke-dasharray="upArcD"
          style="transition:stroke-dasharray 0.3s ease"/>
        <text x="60" y="56" text-anchor="middle" :fill="liveUp > 0 ? '#a78bfa' : '#334155'"
          font-size="18" font-weight="700" font-family="monospace">
          {{ liveUp > 0 ? liveUp : (phase === 'upload' ? '…' : '—') }}
        </text>
        <text x="60" y="68" text-anchor="middle" fill="#475569" font-size="8" font-family="monospace">Mbps</text>
      </svg>
      <div style="font-size:11px;color:#a78bfa;font-weight:600">
        {{ results.upload !== null ? results.upload + ' Mbps peak' : (phase === 'upload' ? 'Measuring…' : '') }}
      </div>
    </div>
  </div>

  <!-- ── Progress bar ────────────────────────────────────────────────────── -->
  <div v-if="phase !== 'idle'" style="flex-shrink:0">
    <div style="display:flex;justify-content:space-between;font-size:11px;color:#475569;margin-bottom:5px">
      <span>{{ phaseLabel }}</span>
      <span>{{ progressPct }}%</span>
    </div>
    <div style="height:4px;background:#1e3a5f;border-radius:2px;overflow:hidden">
      <div :style="`height:100%;background:${progressColor};border-radius:2px;width:${progressPct}%;transition:width 0.3s ease`"></div>
    </div>
  </div>

  <!-- ── Live chart ──────────────────────────────────────────────────────── -->
  <div style="flex:1;background:#0d1b2a;border-radius:8px;border:1px solid #1e3a5f;overflow:hidden;position:relative;min-height:100px">
    <div v-if="!samples.length" style="position:absolute;inset:0;display:flex;align-items:center;justify-content:center;color:#334155;font-size:13px">
      Press Start Test to measure your connection speed
    </div>
    <svg v-else viewBox="0 0 600 140" preserveAspectRatio="none" style="width:100%;height:100%;display:block">
      <defs>
        <linearGradient id="dl-grad" x1="0" y1="0" x2="0" y2="1">
          <stop offset="0%" stop-color="#38bdf8" stop-opacity="0.3"/>
          <stop offset="100%" stop-color="#38bdf8" stop-opacity="0"/>
        </linearGradient>
        <linearGradient id="ul-grad" x1="0" y1="0" x2="0" y2="1">
          <stop offset="0%" stop-color="#a78bfa" stop-opacity="0.3"/>
          <stop offset="100%" stop-color="#a78bfa" stop-opacity="0"/>
        </linearGradient>
      </defs>
      <!-- Grid -->
      <line v-for="g in chartGridLines" :key="g.y" x1="40" :y1="g.y" x2="600" :y2="g.y"
        stroke="#1e3a5f" stroke-width="0.7"/>
      <text v-for="g in chartGridLines" :key="'l'+g.y" x="36" :y="g.y + 3"
        text-anchor="end" fill="#334155" font-size="8" font-family="monospace">{{ g.label }}</text>
      <!-- Download area + line -->
      <path v-if="dlAreaPath" :d="dlAreaPath" fill="url(#dl-grad)"/>
      <path v-if="dlLinePath" :d="dlLinePath" fill="none" stroke="#38bdf8" stroke-width="1.5" stroke-linejoin="round"/>
      <!-- Upload area + line -->
      <path v-if="ulAreaPath" :d="ulAreaPath" fill="url(#ul-grad)"/>
      <path v-if="ulLinePath" :d="ulLinePath" fill="none" stroke="#a78bfa" stroke-width="1.5" stroke-linejoin="round"/>
    </svg>
  </div>

  <!-- ── Legend ──────────────────────────────────────────────────────────── -->
  <div style="display:flex;gap:20px;flex-shrink:0;font-size:11px;color:#475569;padding:0 2px">
    <span><span style="color:#38bdf8">●</span> Download (Mbps)</span>
    <span><span style="color:#a78bfa">●</span> Upload (Mbps)</span>
    <span v-if="results.server" style="margin-left:auto;color:#334155">via {{ results.server }}</span>
  </div>

</div>
</div>
</template>

<script setup>
import { ref, computed } from 'vue'

// ── State ────────────────────────────────────────────────────────────────────
const phase   = ref('idle')    // idle | ping | download | upload | done
const results = ref({ ping: null, download: null, upload: null, server: null })
const samples = ref([])        // { t, dl, ul }  — live chart data
let   liveDown = ref(0)
let   liveUp   = ref(0)

// ── Cloudflare Speed Test endpoints ──────────────────────────────────────────
const CF_DOWN = 'https://speed.cloudflare.com/__down?bytes='
const CF_UP   = 'https://speed.cloudflare.com/__up'
const CF_PING = 'https://speed.cloudflare.com/__down?bytes=0'

// ── Phase labels / progress ───────────────────────────────────────────────────
const PHASES = ['idle', 'ping', 'download', 'upload', 'done']
const progressPct = computed(() => {
  const idx = PHASES.indexOf(phase.value)
  if (idx <= 0) return 0
  if (phase.value === 'done') return 100
  return Math.round(((idx - 1) / 3) * 100)
})
const phaseLabel = computed(() => ({
  ping: 'Measuring latency…', download: 'Measuring download speed…',
  upload: 'Measuring upload speed…', done: 'Complete'
})[phase.value] ?? '')
const progressColor = computed(() => ({
  ping: '#f59e0b', download: '#38bdf8', upload: '#a78bfa', done: '#22c55e'
})[phase.value] ?? '#38bdf8')

// ── Gauge arc helpers ─────────────────────────────────────────────────────────
const ARC_LEN = 157.08   // half circumference of r=50 arc (π * 50)

function arcDasharray(value, maxVal) {
  const frac = Math.min(Math.max(value / maxVal, 0), 1)
  return `${frac * ARC_LEN} ${ARC_LEN}`
}

const pingColor = computed(() => {
  const p = results.value.ping
  if (p === null) return '#334155'
  return p < 30 ? '#22c55e' : p < 80 ? '#f59e0b' : '#ef4444'
})
const pingLabel = computed(() => {
  const p = results.value.ping
  if (p === null) return ''
  return p < 30 ? 'Excellent' : p < 80 ? 'Good' : p < 150 ? 'Fair' : 'Poor'
})

const downArcD = computed(() => arcDasharray(liveDown.value, 500))
const upArcD   = computed(() => arcDasharray(liveUp.value,   200))

// ── Chart computeds ───────────────────────────────────────────────────────────
const CH_W = 600, CH_H = 140, CH_PL = 40, CH_PB = 8, CH_PT = 8
const CH_IW = CH_W - CH_PL
const CH_IH = CH_H - CH_PB - CH_PT

const maxSample = computed(() => {
  const vals = samples.value.flatMap(s => [s.dl, s.ul]).filter(v => v > 0)
  return vals.length ? Math.max(10, Math.max(...vals) * 1.2) : 100
})

const chartGridLines = computed(() => {
  const m = maxSample.value
  return [0.25, 0.5, 0.75, 1].map(f => ({
    y:     CH_PT + CH_IH - f * CH_IH,
    label: Math.round(m * f),
  }))
})

function sampleX(i) { return CH_PL + (i / Math.max(samples.value.length - 1, 1)) * CH_IW }
function sampleY(v) { return CH_PT + CH_IH - Math.min(v / maxSample.value, 1) * CH_IH }

const dlLinePath = computed(() => {
  let d = ''
  samples.value.forEach((s, i) => {
    if (s.dl <= 0) return
    d += (d === '' ? 'M' : 'L') + `${sampleX(i)},${sampleY(s.dl)} `
  })
  return d || null
})

const dlAreaPath = computed(() => {
  const base = CH_PT + CH_IH
  const pts  = samples.value.map((s, i) => s.dl > 0 ? { x: sampleX(i), y: sampleY(s.dl) } : null).filter(Boolean)
  if (pts.length < 2) return null
  return `M${pts[0].x},${base} ` + pts.map(p => `L${p.x},${p.y}`).join(' ') + ` L${pts[pts.length-1].x},${base} Z`
})

const ulLinePath = computed(() => {
  let d = ''
  samples.value.forEach((s, i) => {
    if (s.ul <= 0) return
    d += (d === '' ? 'M' : 'L') + `${sampleX(i)},${sampleY(s.ul)} `
  })
  return d || null
})

const ulAreaPath = computed(() => {
  const base = CH_PT + CH_IH
  const pts  = samples.value.map((s, i) => s.ul > 0 ? { x: sampleX(i), y: sampleY(s.ul) } : null).filter(Boolean)
  if (pts.length < 2) return null
  return `M${pts[0].x},${base} ` + pts.map(p => `L${p.x},${p.y}`).join(' ') + ` L${pts[pts.length-1].x},${base} Z`
})

// ── Test logic ────────────────────────────────────────────────────────────────

async function measurePing(attempts = 5) {
  const latencies = []
  for (let i = 0; i < attempts; i++) {
    const t0  = performance.now()
    try { await fetch(CF_PING, { cache: 'no-store' }) } catch { /* ignore */ }
    latencies.push(Math.round(performance.now() - t0))
  }
  // Drop highest outlier, return median
  latencies.sort((a, b) => a - b)
  return latencies[Math.floor(latencies.length / 2)]
}

async function measureDownload() {
  // Download sizes in bytes for progressive measurement
  const sizes   = [500_000, 2_000_000, 10_000_000, 25_000_000]
  let   peakMbps = 0
  const t0 = performance.now()

  for (const bytes of sizes) {
    const start = performance.now()
    try {
      const res  = await fetch(CF_DOWN + bytes, { cache: 'no-store' })
      await res.arrayBuffer()
      const secs  = (performance.now() - start) / 1000
      const mbps  = Math.round((bytes * 8) / secs / 1_000_000)
      peakMbps    = Math.max(peakMbps, mbps)
      liveDown.value = mbps
      samples.value.push({ t: performance.now() - t0, dl: mbps, ul: 0 })
    } catch { break }
  }
  return peakMbps
}

async function measureUpload() {
  const sizes   = [500_000, 2_000_000, 5_000_000]
  let   peakMbps = 0
  const t0       = samples.value.length ? samples.value[samples.value.length - 1].t : 0

  for (const bytes of sizes) {
    const body  = new Uint8Array(bytes).fill(0x61)
    const start = performance.now()
    try {
      await fetch(CF_UP, { method: 'POST', body, cache: 'no-store' })
      const secs  = (performance.now() - start) / 1000
      const mbps  = Math.round((bytes * 8) / secs / 1_000_000)
      peakMbps    = Math.max(peakMbps, mbps)
      liveUp.value = mbps
      samples.value.push({ t: t0 + performance.now() - start, dl: 0, ul: mbps })
    } catch { break }
  }
  return peakMbps
}

async function startTest() {
  if (phase.value !== 'idle' && phase.value !== 'done') return
  results.value = { ping: null, download: null, upload: null, server: 'speed.cloudflare.com' }
  samples.value = []
  liveDown.value = 0
  liveUp.value   = 0

  phase.value            = 'ping'
  results.value.ping     = await measurePing()

  phase.value            = 'download'
  results.value.download = await measureDownload()

  phase.value            = 'upload'
  results.value.upload   = await measureUpload()

  liveDown.value = results.value.download
  liveUp.value   = results.value.upload
  phase.value    = 'done'
}
</script>
