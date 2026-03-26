
<template>
  <div id="map" style="flex:1;background:#0a0f1e"></div>
</template>

<script setup>
import { onMounted, onUnmounted, watch } from 'vue'
import L from 'leaflet'

const props  = defineProps({ data: Object, showCables: Boolean, showCableRoute: Boolean })
const emit   = defineEmits(['hopChange'])

let map, packetMarker, routeLayer, cablesLayer, cableRouteLayer
let animInterval = null
let coords       = []   // [[lat, lon], ...]
let stepIndex    = 0
let cablesGeojson = null  // kept for cable-route matching

// ── Helpers ──────────────────────────────────────────────────────────────────

function haversine(lat1, lon1, lat2, lon2) {
  const R    = 6371
  const dLat = (lat2 - lat1) * Math.PI / 180
  const dLon = (lon2 - lon1) * Math.PI / 180
  const a    = Math.sin(dLat / 2) ** 2 +
    Math.cos(lat1 * Math.PI / 180) * Math.cos(lat2 * Math.PI / 180) * Math.sin(dLon / 2) ** 2
  return R * 2 * Math.atan2(Math.sqrt(a), Math.sqrt(1 - a))
}

/** Generate interpolated arc points between two lat/lon pairs.
 *  The arc bulges perpendicular-left of the travel direction so it
 *  always curves the same way regardless of which direction the route goes.
 *  For very short hops (< 200 km) the arc is nearly flat.
 */
function arcPoints(lat1, lon1, lat2, lon2, steps = 40) {
  const dist = haversine(lat1, lon1, lat2, lon2)

  // Flat for short hops; gentle curve for long ones; capped at 10°
  const arcHeight = Math.min(dist / 50, 10)

  // Direction vector (in lat/lon space)
  const dLat = lat2 - lat1
  const dLon = lon2 - lon1

  // Perpendicular-left unit vector (rotate 90° CCW): (-dLon, dLat)
  const len = Math.sqrt(dLat * dLat + dLon * dLon) || 1
  const perpLat = -dLon / len
  const perpLon =  dLat / len

  const pts = []
  for (let i = 0; i <= steps; i++) {
    const t      = i / steps
    const lat    = lat1 + dLat * t
    const lon    = lon1 + dLon * t
    const offset = Math.sin(Math.PI * t) * arcHeight
    pts.push([lat + perpLat * offset, lon + perpLon * offset])
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

// ── Cable route proximity matching ────────────────────────────────────────────

/** Min distance from a point to any vertex of a cable's coordinate list. */
function minDistToCable(lat, lon, cableCoords) {
  let minD = Infinity
  for (const [clon, clat] of cableCoords) {
    const d = haversine(lat, lon, clat, clon)
    if (d < minD) minD = d
  }
  return minD
}

/**
 * Find the cable in geojson whose path most closely connects h1→h2.
 * Scores each cable by summing min-distance from hop1, hop2, and midpoint.
 * Returns null if no cable is within a reasonable threshold.
 */
function findNearestCable(lat1, lon1, lat2, lon2, geojson) {
  const midLat = (lat1 + lat2) / 2
  const midLon = (lon1 + lon2) / 2
  let best = null
  let bestScore = Infinity

  for (const feature of geojson.features) {
    if (!feature.geometry) continue
    let cableCoords = null
    if (feature.geometry.type === 'LineString') {
      cableCoords = feature.geometry.coordinates
    } else if (feature.geometry.type === 'MultiLineString') {
      cableCoords = feature.geometry.coordinates.flat()
    }
    if (!cableCoords || cableCoords.length === 0) continue

    const score = minDistToCable(lat1, lon1, cableCoords)
                + minDistToCable(lat2, lon2, cableCoords)
                + minDistToCable(midLat, midLon, cableCoords)

    if (score < bestScore) {
      bestScore = score
      best = { feature, score }
    }
  }

  // Accept only if average per-point distance < 800 km
  return best && bestScore / 3 < 800 ? best : null
}

/** Draw (or clear) the cable-route highlight layer. */
function drawCableRoute() {
  if (cableRouteLayer) { cableRouteLayer.remove(); cableRouteLayer = null }
  if (!props.showCableRoute || !cablesGeojson || !props.data) return

  cableRouteLayer = L.layerGroup().addTo(map)
  const hops = props.data.hops
  const seen = new Set()

  for (let i = 1; i < hops.length; i++) {
    const h1 = hops[i - 1]
    const h2 = hops[i]
    const dist = haversine(h1.lat, h1.lon, h2.lat, h2.lon)
    if (dist < 500) continue   // short hops don't need undersea cables

    const match = findNearestCable(h1.lat, h1.lon, h2.lat, h2.lon, cablesGeojson)
    if (!match) continue

    const name = match.feature.properties?.name
               || match.feature.properties?.cable
               || 'Unknown cable'
    if (seen.has(name)) continue
    seen.add(name)

    // Draw glow + highlight line for matched cable
    L.geoJSON(match.feature, {
      style: { color: '#f59e0b', weight: 5, opacity: 0.15 }
    }).addTo(cableRouteLayer)

    L.geoJSON(match.feature, {
      style: { color: '#fbbf24', weight: 2.5, opacity: 0.9 }
    })
      .bindPopup(`<b>${name}</b><br><span style="color:#94a3b8">Nearest submarine cable</span>`)
      .addTo(cableRouteLayer)
  }
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
    const res  = await fetch('/data/cables.geojson')
    cablesGeojson = await res.json()
    cablesLayer   = L.geoJSON(cablesGeojson, {
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
  drawCableRoute()
  replay()
})

watch(() => props.showCables, show => {
  if (!cablesLayer) return
  show ? cablesLayer.addTo(map) : cablesLayer.remove()
})

watch(() => props.showCableRoute, () => {
  drawCableRoute()
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
