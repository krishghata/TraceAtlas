
<template>
  <div id="map" style="flex:1;background:#0a0f1e"></div>
</template>

<script setup>
import { onMounted, onUnmounted, watch } from 'vue'
import L from 'leaflet'

const props  = defineProps({ data: Object, showCables: Boolean })
const emit   = defineEmits(['hopChange'])

let map, packetMarker, routeLayer, cablesLayer
let animInterval = null
let coords       = []   // [[lat, lon], ...]
let stepIndex    = 0

// ── Helpers ──────────────────────────────────────────────────────────────────

function haversine(lat1, lon1, lat2, lon2) {
  const R    = 6371
  const dLat = (lat2 - lat1) * Math.PI / 180
  const dLon = (lon2 - lon1) * Math.PI / 180
  const a    = Math.sin(dLat / 2) ** 2 +
    Math.cos(lat1 * Math.PI / 180) * Math.cos(lat2 * Math.PI / 180) * Math.sin(dLon / 2) ** 2
  return R * 2 * Math.atan2(Math.sqrt(a), Math.sqrt(1 - a))
}

/** Generate interpolated arc points between two lat/lon pairs. */
function arcPoints(lat1, lon1, lat2, lon2, steps = 40) {
  const dist      = haversine(lat1, lon1, lat2, lon2)
  const arcHeight = Math.min(dist / 30, 18)  // scale lift with distance, cap at 18°
  const pts       = []
  for (let i = 0; i <= steps; i++) {
    const t      = i / steps
    const lat    = lat1 + (lat2 - lat1) * t
    const lon    = lon1 + (lon2 - lon1) * t
    const offset = Math.sin(Math.PI * t) * arcHeight
    pts.push([lat + offset, lon])
  }
  return pts
}

/** Color a hop marker green→yellow→red based on its latency. */
function latencyColor(latency, min, max) {
  if (latency == null) return '#64748b'
  const t = max === min ? 0 : Math.max(0, Math.min(1, (latency - min) / (max - min)))
  if (t < 0.5) {
    const r = Math.round(t * 2 * 255)
    return `rgb(${r},210,60)`
  }
  const g = Math.round((1 - (t - 0.5) * 2) * 210)
  return `rgb(255,${g},40)`
}

// ── Lifecycle ─────────────────────────────────────────────────────────────────

onMounted(async () => {
  map = L.map('map', { zoomControl: true }).setView([20, 0], 2)
  L.tileLayer('https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png', {
    attribution: '© OpenStreetMap contributors',
    className: 'map-tiles'
  }).addTo(map)

  // Load submarine cable overlay
  try {
    const res     = await fetch('/data/cables.geojson')
    const geojson = await res.json()
    cablesLayer   = L.geoJSON(geojson, {
      style: { color: '#f97316', weight: 1.2, opacity: 0.45 }
    })
    if (props.showCables) cablesLayer.addTo(map)
  } catch (_) {
    console.warn('cables.geojson not found — run: node backend/scripts/download-cables.js')
  }
})

onUnmounted(() => {
  if (animInterval) clearInterval(animInterval)
  if (map) map.remove()
})

// ── Watchers ──────────────────────────────────────────────────────────────────

watch(() => props.data, d => {
  if (!d) return
  coords = d.hops.map(h => [h.lat, h.lon])
  stepIndex = 0
  drawRoute()
  replay()
})

watch(() => props.showCables, show => {
  if (!cablesLayer) return
  show ? cablesLayer.addTo(map) : cablesLayer.remove()
})

// ── Drawing ───────────────────────────────────────────────────────────────────

function drawRoute() {
  if (routeLayer) map.removeLayer(routeLayer)
  routeLayer = L.layerGroup().addTo(map)

  const hops      = props.data.hops
  const latencies = hops.map(h => h.latency).filter(v => v != null)
  const minLat    = Math.min(...latencies)
  const maxLat    = Math.max(...latencies)

  // Build full arc path (all segments joined)
  const fullArcPts = []
  for (let i = 1; i < coords.length; i++) {
    const seg = arcPoints(coords[i - 1][0], coords[i - 1][1], coords[i][0], coords[i][1])
    if (i > 1) seg.shift()  // avoid duplicate junction point
    fullArcPts.push(...seg)
  }

  if (fullArcPts.length > 1) {
    // Glow layers: outer → inner
    [
      { weight: 12, opacity: 0.04, color: '#38bdf8' },
      { weight: 7,  opacity: 0.1,  color: '#38bdf8' },
      { weight: 4,  opacity: 0.3,  color: '#38bdf8' },
      { weight: 2,  opacity: 0.75, color: '#38bdf8' },
      { weight: 1,  opacity: 1.0,  color: '#bae6fd' }  // bright core
    ].forEach(o => L.polyline(fullArcPts, o).addTo(routeLayer))
  }

  // Hop markers (latency heatmap)
  hops.forEach(h => {
    const color = latencyColor(h.latency, minLat, maxLat)
    L.circleMarker([h.lat, h.lon], {
      radius: 6, color, fillColor: color, fillOpacity: 0.9, weight: 2
    })
      .bindPopup(
        `<b>Hop ${h.hop}</b><br>` +
        `${h.country || '?'} — ${h.org || 'unknown'}<br>` +
        `${h.ip}<br>` +
        (h.latency != null ? `${h.latency}ms` : 'timeout')
      )
      .addTo(routeLayer)
  })

  // Auto-zoom
  if (fullArcPts.length > 1) {
    map.fitBounds(L.latLngBounds(fullArcPts), { padding: [40, 40] })
  }
}

// ── Animation ─────────────────────────────────────────────────────────────────

/** Moves the packet marker to the given hop index (0-based). */
function moveMarkerTo(idx) {
  const pt = coords[idx]
  if (!packetMarker) {
    packetMarker = L.circleMarker(pt, {
      radius: 8, color: '#f97316', fillColor: '#f97316', fillOpacity: 1, weight: 0
    }).addTo(map)
  } else {
    packetMarker.setLatLng(pt)
  }
  stepIndex = idx
  emit('hopChange', idx)
}

function replay() {
  if (!coords.length) return
  if (animInterval) clearInterval(animInterval)
  moveMarkerTo(0)
  let i = 0
  animInterval = setInterval(() => {
    i++
    if (i >= coords.length) { clearInterval(animInterval); animInterval = null; return }
    moveMarkerTo(i)
  }, 500)
}

function stepNext() {
  if (!coords.length) return
  if (animInterval) { clearInterval(animInterval); animInterval = null }
  moveMarkerTo(Math.min(stepIndex + 1, coords.length - 1))
}

function stepPrev() {
  if (!coords.length) return
  if (animInterval) { clearInterval(animInterval); animInterval = null }
  moveMarkerTo(Math.max(stepIndex - 1, 0))
}

defineExpose({ replay, stepNext, stepPrev })
</script>

<style>
.map-tiles { filter: brightness(0.7) saturate(0.6) hue-rotate(200deg); }
</style>
