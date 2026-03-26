
<template>
<div id="traceatlas-app" style="background:#0a0f1e;flex:1;color:white;font-family:'Segoe UI',sans-serif;display:flex;flex-direction:column;overflow:hidden">

  <!-- ── Toolbar ──────────────────────────────────────────────────────────── -->
  <div style="padding:10px 14px;display:flex;align-items:center;gap:8px;background:#0d1b2a;border-bottom:1px solid #1e3a5f;flex-shrink:0">
    <span style="font-weight:700;color:#38bdf8;font-size:15px;margin-right:4px">TraceAtlas</span>

    <input
      v-model="target"
      placeholder="Domain or IP — e.g. google.com"
      @keyup.enter="trace"
      style="flex:1;padding:7px 12px;background:#0a0f1e;border:1px solid #1e3a5f;border-radius:6px;color:white;font-size:13px;outline:none"
    />

    <button @click="trace" :disabled="loading"
      style="padding:7px 16px;background:#1d4ed8;color:white;border:none;border-radius:6px;font-size:13px;font-weight:600;cursor:pointer;white-space:nowrap">
      {{ loading ? 'Tracing…' : 'Trace' }}
    </button>

    <button @click="replayAnim" :disabled="!data"
      style="padding:7px 12px;background:#1e293b;color:#94a3b8;border:1px solid #334155;border-radius:6px;font-size:13px;cursor:pointer">
      ▶ Replay
    </button>

    <template v-if="data">
      <button @click="prevHop"
        style="padding:7px 10px;background:#1e293b;color:#94a3b8;border:1px solid #334155;border-radius:6px;font-size:13px;cursor:pointer">◀</button>
      <span style="font-size:12px;color:#475569;white-space:nowrap">hop {{ currentHop + 1 }} / {{ data.hops.length }}</span>
      <button @click="nextHop"
        style="padding:7px 10px;background:#1e293b;color:#94a3b8;border:1px solid #334155;border-radius:6px;font-size:13px;cursor:pointer">▶</button>
    </template>

    <label style="font-size:12px;color:#64748b;display:flex;align-items:center;gap:4px;white-space:nowrap;cursor:pointer">
      <input type="checkbox" v-model="showCables" style="accent-color:#38bdf8"/> Undersea Cables
    </label>
  </div>

  <!-- ── Error banner ─────────────────────────────────────────────────────── -->
  <div v-if="error"
    style="padding:8px 16px;background:#450a0a;color:#fca5a5;font-size:13px;border-bottom:1px solid #7f1d1d;flex-shrink:0">
    ⚠ {{ error }}
  </div>

  <!-- ── Summary bar ──────────────────────────────────────────────────────── -->
  <SummaryPanel :data="data" />

  <!-- ── Main content ─────────────────────────────────────────────────────── -->
  <div style="display:flex;flex:1;overflow:hidden">
    <HopList :hops="data?.hops" />
    <MapView :data="data" :showCables="showCables" :showCableRoute="showCableRoute" ref="mapRef" @hopChange="currentHop = $event" />
    <InsightsPanel :insights="data?.insights" />
  </div>

</div>
</template>

<script setup>
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import html2canvas from 'html2canvas'

import MapView       from './MapView.vue'
import HopList       from './HopList.vue'
import InsightsPanel from './InsightsPanel.vue'
import SummaryPanel  from './SummaryPanel.vue'

import { parseTraceroute, isPrivateIP, deduplicateHops } from '../lib/traceroute.js'
import { batchResolveGeo, getSourceLocation }            from '../lib/geo.js'
import { generateInsights }                              from '../lib/insights.js'
import { getDb }                                         from '../lib/db.js'

/** Snap consecutive hops that share the same country+org to their median position.
 *  Eliminates zigzag arcs caused by ISP IPs geo-resolving to different cities. */
function smoothGeoOutliers(hops) {
  let i = 0
  while (i < hops.length) {
    let j = i + 1
    while (j < hops.length &&
           hops[j].country === hops[i].country &&
           hops[j].org     === hops[i].org) j++
    if (j - i > 1) {
      const group = hops.slice(i, j)
      const lats  = group.map(h => h.lat).sort((a, b) => a - b)
      const lons  = group.map(h => h.lon).sort((a, b) => a - b)
      const mid   = Math.floor(group.length / 2)
      const mLat  = lats[mid]
      const mLon  = lons[mid]
      for (let k = i; k < j; k++) {
        hops[k].lat = mLat
        hops[k].lon = mLon
      }
    }
    i = j
  }
}

const data            = ref(null)
const mapRef          = ref(null)
const showCables      = ref(false)
const showCableRoute  = computed(() => showCables.value)
const target     = ref('')
const loading    = ref(false)
const error      = ref(null)
const currentHop = ref(0)

async function trace() {
  const t = target.value.trim()
  if (!t) return

  loading.value    = true
  error.value      = null
  data.value       = null
  currentHop.value = 0

  try {
    const db = await getDb()

    // ── Check trace cache (1-hour TTL) ────────────────────────────────────
    const cached = await db.select(
      'SELECT result, updated_at FROM trace_cache WHERE target = ?', [t]
    )
    if (cached[0] && (Date.now() - cached[0].updated_at) < 60 * 60 * 1000) {
      data.value = JSON.parse(cached[0].result)
      return
    }

    // ── Run traceroute via Tauri (executes on user's machine) ─────────────
    const traceResult = await invoke('run_traceroute', { target: t })

    let hops = parseTraceroute(traceResult.output, traceResult.is_win)
    hops = hops.filter(h => !isPrivateIP(h.ip))
    hops = deduplicateHops(hops)

    if (!hops.length) throw new Error('Traceroute returned no public hops — target may be unreachable')

    // ── Geo-enrich + source location in parallel ──────────────────────────
    const [sourceLocation, geoMap] = await Promise.all([
      getSourceLocation(),
      batchResolveGeo(hops.map(h => h.ip))
    ])

    const enrichedHops = hops
      .map(h => {
        const geo = geoMap.get(h.ip)
        if (!geo) return null
        return { hop: h.hop, ip: h.ip, lat: geo.lat, lon: geo.lon, country: geo.country, org: geo.org, latency: h.latency }
      })
      .filter(Boolean)

    // Smooth geo outliers: group consecutive hops with the same country+org
    // and snap them all to the median lat/lon of the group.
    // This prevents zigzag paths when an ISP's IPs geo-resolve to different cities.
    smoothGeoOutliers(enrichedHops)

    if (!enrichedHops.length) throw new Error('Could not geo-locate any hops')

    const insights = generateInsights(enrichedHops)
    const result   = {
      source:      sourceLocation,
      hops:        enrichedHops,
      destination: { ip: enrichedHops[enrichedHops.length - 1].ip },
      insights
    }

    // Cache the trace result
    await db.execute(
      'INSERT OR REPLACE INTO trace_cache (target, result, updated_at) VALUES (?, ?, ?)',
      [t, JSON.stringify(result), Date.now()]
    )

    data.value = result
  } catch (e) {
    error.value = e.message || String(e)
  } finally {
    loading.value = false
  }
}

function replayAnim() { currentHop.value = 0; mapRef.value?.replay() }
function prevHop()    { mapRef.value?.stepPrev() }
function nextHop()    { mapRef.value?.stepNext() }

function exportJSON() {
  if (!data.value) return
  const blob = new Blob([JSON.stringify(data.value, null, 2)], { type: 'application/json' })
  const url  = URL.createObjectURL(blob)
  Object.assign(document.createElement('a'), {
    href: url, download: `traceatlas-${target.value}-${Date.now()}.json`
  }).click()
  URL.revokeObjectURL(url)
}

async function screenshot() {
  try {
    const canvas = await html2canvas(document.getElementById('traceatlas-app'), {
      useCORS: true, allowTaint: true, backgroundColor: '#0a0f1e'
    })
    canvas.toBlob(blob => {
      const url = URL.createObjectURL(blob)
      Object.assign(document.createElement('a'), {
        href: url, download: `traceatlas-${target.value || 'screenshot'}-${Date.now()}.png`
      }).click()
      URL.revokeObjectURL(url)
    })
  } catch (e) { console.error('Screenshot failed:', e) }
}
</script>
