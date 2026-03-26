
<template>
<div style="flex:1;display:flex;flex-direction:column;overflow:hidden;padding:12px 14px;gap:10px;box-sizing:border-box;background:#0a0f1e;color:white;font-family:'Segoe UI',sans-serif">

  <!-- ── Header ──────────────────────────────────────────────────────────── -->
  <div style="flex-shrink:0">
    <h2 style="margin:0 0 2px;font-size:15px;font-weight:700;color:#e2e8f0">MTR — My Traceroute</h2>
    <p style="margin:0;font-size:12px;color:#475569">Continuous per-hop latency and packet loss — combines traceroute + ping</p>
  </div>

  <!-- ── Controls ───────────────────────────────────────────────────────── -->
  <div style="display:flex;gap:8px;align-items:center;flex-shrink:0;flex-wrap:wrap">
    <input v-model="target" @keyup.enter="toggle" placeholder="Target — e.g. google.com or 8.8.8.8"
      style="flex:1;min-width:160px;padding:7px 12px;background:#0d1b2a;border:1px solid #1e3a5f;border-radius:6px;color:white;font-size:13px;outline:none"
      :disabled="phase === 'probing'"/>

    <select v-model="timeoutMs" :disabled="phase !== 'idle'"
      style="padding:7px 10px;background:#0d1b2a;border:1px solid #1e3a5f;border-radius:6px;color:#94a3b8;font-size:12px">
      <option :value="800">Timeout 800ms</option>
      <option :value="1500">Timeout 1.5s</option>
      <option :value="3000">Timeout 3s</option>
    </select>

    <button @click="toggle"
      :style="`padding:7px 18px;border:none;border-radius:6px;font-size:13px;font-weight:600;cursor:pointer;color:white;
        background:${phase==='probing'?'#7f1d1d':phase==='tracing'?'#1e3a5f':'#1d4ed8'}`">
      {{ phase === 'idle' ? '▶ Start' : phase === 'tracing' ? 'Discovering hops…' : '⏹ Stop' }}
    </button>

    <button @click="resetData" :disabled="phase !== 'idle' || !hops.length"
      style="padding:7px 12px;background:#1e293b;color:#94a3b8;border:1px solid #334155;border-radius:6px;font-size:12px;cursor:pointer">
      Reset
    </button>

    <!-- Live stats -->
    <div v-if="phase === 'probing'" style="display:flex;align-items:center;gap:6px;margin-left:auto">
      <span style="width:8px;height:8px;border-radius:50%;background:#22c55e;box-shadow:0 0 8px #22c55e;animation:pulse 1s infinite;display:inline-block"/>
      <span style="font-size:11px;color:#64748b">Round {{ round }} · {{ hops.length }} hops</span>
    </div>
  </div>

  <!-- ── Error ──────────────────────────────────────────────────────────── -->
  <div v-if="error" style="padding:8px 14px;background:#450a0a;color:#fca5a5;font-size:12px;border-radius:6px;border:1px solid #7f1d1d;flex-shrink:0">
    ⚠ {{ error }}
  </div>

  <!-- ── Empty state ────────────────────────────────────────────────────── -->
  <div v-if="phase === 'idle' && !hops.length && !error"
    style="flex:1;display:flex;align-items:center;justify-content:center;color:#334155;font-size:13px">
    Enter a target and press Start
  </div>

  <!-- ── Hop table ───────────────────────────────────────────────────────── -->
  <div v-if="hops.length" style="flex:1;overflow-y:auto;min-height:0">
    <table style="width:100%;border-collapse:collapse;font-size:12px">
      <thead>
        <tr style="background:#0a1628;border-bottom:2px solid #1e3a5f">
          <th style="padding:7px 10px;text-align:left;font-size:10px;color:#475569;font-weight:600;text-transform:uppercase;letter-spacing:0.07em;width:32px">#</th>
          <th style="padding:7px 10px;text-align:left;font-size:10px;color:#475569;font-weight:600;text-transform:uppercase;letter-spacing:0.07em">Host</th>
          <th style="padding:7px 10px;text-align:right;font-size:10px;color:#ef4444;font-weight:600;text-transform:uppercase;letter-spacing:0.07em;width:55px">Loss%</th>
          <th style="padding:7px 10px;text-align:right;font-size:10px;color:#475569;font-weight:600;text-transform:uppercase;letter-spacing:0.07em;width:45px">Snt</th>
          <th style="padding:7px 10px;text-align:right;font-size:10px;color:#94a3b8;font-weight:600;text-transform:uppercase;letter-spacing:0.07em;width:60px">Last</th>
          <th style="padding:7px 10px;text-align:right;font-size:10px;color:#38bdf8;font-weight:600;text-transform:uppercase;letter-spacing:0.07em;width:60px">Avg</th>
          <th style="padding:7px 10px;text-align:right;font-size:10px;color:#22c55e;font-weight:600;text-transform:uppercase;letter-spacing:0.07em;width:60px">Best</th>
          <th style="padding:7px 10px;text-align:right;font-size:10px;color:#f59e0b;font-weight:600;text-transform:uppercase;letter-spacing:0.07em;width:60px">Wrst</th>
          <th style="padding:7px 10px;text-align:right;font-size:10px;color:#475569;font-weight:600;text-transform:uppercase;letter-spacing:0.07em;width:60px">StDev</th>
          <th style="padding:7px 10px;text-align:left;font-size:10px;color:#475569;font-weight:600;text-transform:uppercase;letter-spacing:0.07em;width:100px">Graph</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="(hop, i) in hops" :key="hop.ip"
          :style="`border-bottom:1px solid #0f1f35;${i%2===0?'background:#0a0f1e':'background:#0d1b2a'};transition:background 0.1s`">

          <!-- # -->
          <td style="padding:6px 10px;color:#334155;font-family:monospace">{{ i + 1 }}</td>

          <!-- Host -->
          <td style="padding:6px 10px;font-family:monospace">
            <span :style="`color:${hop.stats.loss===100?'#475569':rttColor(hop.stats.avg)}`">
              {{ hop.ip }}
            </span>
            <span v-if="hop.stats.loss===100" style="font-size:10px;color:#475569;margin-left:6px">(no reply)</span>
          </td>

          <!-- Loss% -->
          <td style="padding:6px 10px;text-align:right;font-family:monospace">
            <span :style="`font-weight:700;color:${lossColor(hop.stats.loss)}`">
              {{ hop.stats.loss }}%
            </span>
          </td>

          <!-- Sent -->
          <td style="padding:6px 10px;text-align:right;color:#475569;font-family:monospace">{{ hop.stats.sent }}</td>

          <!-- Last -->
          <td style="padding:6px 10px;text-align:right;font-family:monospace">
            <span :style="`color:${hop.stats.last!=null?rttColor(hop.stats.last):'#334155'}`">
              {{ hop.stats.last != null ? hop.stats.last + 'ms' : '—' }}
            </span>
          </td>

          <!-- Avg -->
          <td style="padding:6px 10px;text-align:right;font-family:monospace">
            <span :style="`color:${hop.stats.avg!=null?rttColor(hop.stats.avg):'#334155'}`">
              {{ hop.stats.avg != null ? hop.stats.avg + 'ms' : '—' }}
            </span>
          </td>

          <!-- Best -->
          <td style="padding:6px 10px;text-align:right;color:#22c55e;font-family:monospace">
            {{ hop.stats.best != null ? hop.stats.best + 'ms' : '—' }}
          </td>

          <!-- Worst -->
          <td style="padding:6px 10px;text-align:right;font-family:monospace">
            <span :style="`color:${hop.stats.worst!=null?rttColor(hop.stats.worst):'#334155'}`">
              {{ hop.stats.worst != null ? hop.stats.worst + 'ms' : '—' }}
            </span>
          </td>

          <!-- StDev -->
          <td style="padding:6px 10px;text-align:right;color:#475569;font-family:monospace">
            {{ hop.stats.stdev != null ? hop.stats.stdev + 'ms' : '—' }}
          </td>

          <!-- Mini sparkline -->
          <td style="padding:4px 10px">
            <svg viewBox="0 0 80 20" style="width:80px;height:20px;display:block">
              <path v-if="sparkPath(hop)" :d="sparkPath(hop)"
                fill="none" :stroke="rttColor(hop.stats.avg)" stroke-width="1.5" stroke-linejoin="round"/>
              <circle v-for="(pt, pi) in sparkDots(hop)" :key="pi"
                :cx="pt.x" :cy="pt.y" r="1.5" :fill="pt.color"/>
            </svg>
          </td>
        </tr>
      </tbody>
    </table>
  </div>

</div>
</template>

<script setup>
import { ref, onUnmounted } from 'vue'
import { invoke }           from '@tauri-apps/api/core'
import { parseTraceroute }  from '../lib/traceroute.js'

// ── State ─────────────────────────────────────────────────────────────────────
const target    = ref('google.com')
const timeoutMs = ref(1500)
const phase     = ref('idle')   // idle | tracing | probing
const error     = ref(null)
const round     = ref(0)
const hops      = ref([])       // { ip, history: number[], stats: {...} }
let   running   = false

// ── Color helpers ─────────────────────────────────────────────────────────────
function rttColor(rtt) {
  if (rtt == null) return '#334155'
  return rtt < 50 ? '#22c55e' : rtt < 150 ? '#f59e0b' : rtt < 300 ? '#f97316' : '#ef4444'
}
function lossColor(loss) {
  return loss === 0 ? '#334155' : loss < 10 ? '#f59e0b' : '#ef4444'
}

// ── Sparkline helpers ─────────────────────────────────────────────────────────
const SPARK_MAX_PTS = 30

function sparkDots(hop) {
  const pts = hop.history.slice(-SPARK_MAX_PTS)
  if (!pts.length) return []
  const valid = pts.filter(v => v != null)
  const max   = valid.length ? Math.max(50, Math.max(...valid) * 1.1) : 200
  const step  = pts.length > 1 ? 80 / (pts.length - 1) : 40
  return pts.map((v, i) => ({
    x:     i * step,
    y:     v == null ? 18 : 18 - (v / max) * 16,
    color: rttColor(v),
  }))
}

function sparkPath(hop) {
  const pts  = hop.history.slice(-SPARK_MAX_PTS)
  const dots = sparkDots(hop)
  let d = ''; let prev = null
  pts.forEach((v, i) => {
    if (v == null) { prev = null; return }
    d += prev === null ? `M${dots[i].x},${dots[i].y} ` : `L${dots[i].x},${dots[i].y} `
    prev = dots[i]
  })
  return d || null
}

// ── Stats calculation ─────────────────────────────────────────────────────────
function calcStats(history) {
  const sent     = history.length
  const received = history.filter(v => v != null)
  const loss     = sent ? Math.round(((sent - received.length) / sent) * 100) : 0
  if (!received.length) return { sent, received: 0, loss, last: null, avg: null, best: null, worst: null, stdev: null }

  const last  = history[history.length - 1] ?? null
  const best  = Math.min(...received)
  const worst = Math.max(...received)
  const avg   = Math.round(received.reduce((a, b) => a + b, 0) / received.length)

  let stdev = null
  if (received.length > 1) {
    const variance = received.reduce((s, v) => s + (v - avg) ** 2, 0) / received.length
    stdev = Math.round(Math.sqrt(variance))
  }
  return { sent, received: received.length, loss, last, avg, best, worst, stdev }
}

// ── MTR logic ─────────────────────────────────────────────────────────────────
async function toggle() {
  if (phase.value === 'probing') {
    running = false
    phase.value = 'idle'
    return
  }
  if (phase.value !== 'idle') return

  const t = target.value.trim()
  if (!t) return

  running     = true
  error.value = null
  round.value = 0

  // Step 1: traceroute to discover hops
  phase.value = 'tracing'
  let hopIps = []
  try {
    const res    = await invoke('run_traceroute', { target: t })
    const parsed = parseTraceroute(res.output, res.is_win)
    hopIps = parsed
      .filter(h => h.ip && h.ip !== '*')
      .map(h => h.ip)
    if (!hopIps.length) throw new Error('No hops discovered — target unreachable')
  } catch (e) {
    error.value = String(e)
    phase.value = 'idle'
    running     = false
    return
  }

  // Initialise hop state
  hops.value = hopIps.map(ip => ({ ip, history: [], stats: calcStats([]) }))
  phase.value = 'probing'

  // Step 2: probe loop
  while (running) {
    try {
      const results = await invoke('mtr_probe', { ips: hopIps, timeoutMs: timeoutMs.value })
      if (!running) break

      round.value++
      results.forEach((res, i) => {
        if (!hops.value[i]) return
        hops.value[i].history.push(res.rtt_ms ?? null)
        hops.value[i].stats = calcStats(hops.value[i].history)
      })
    } catch { /* ignore transient probe errors */ }

    // Wait before next round (remaining time after probe)
    await new Promise(r => setTimeout(r, 500))
  }

  phase.value = 'idle'
}

function resetData() {
  hops.value  = []
  round.value = 0
  error.value = null
}

onUnmounted(() => { running = false })
</script>

<style scoped>
@keyframes pulse {
  0%, 100% { opacity: 1; }
  50%       { opacity: 0.3; }
}
</style>
