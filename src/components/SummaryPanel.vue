
<template>
<div v-if="data" style="padding:10px 16px;background:#0f172a;border-bottom:1px solid #1e3a5f;font-family:sans-serif;color:white;display:flex;align-items:center">
  <div style="display:flex;gap:28px;flex-wrap:wrap;font-size:13px;align-items:center">

    <div>
      <span style="color:#475569;font-size:11px;text-transform:uppercase;letter-spacing:.05em">Hops</span>
      <span style="color:#38bdf8;font-weight:700;margin-left:6px">{{ data.hops.length }}</span>
    </div>

    <div>
      <span style="color:#475569;font-size:11px;text-transform:uppercase;letter-spacing:.05em">Countries</span>
      <span style="color:#38bdf8;font-weight:700;margin-left:6px">{{ countries.join(' → ') }}</span>
    </div>

    <div>
      <span style="color:#475569;font-size:11px;text-transform:uppercase;letter-spacing:.05em">Distance</span>
      <span style="color:#38bdf8;font-weight:700;margin-left:6px">~{{ data.insights?.totalDistance?.toLocaleString() }} km</span>
    </div>

    <div>
      <span style="color:#475569;font-size:11px;text-transform:uppercase;letter-spacing:.05em">Max Latency</span>
      <span :style="`color:${maxLatency > 150 ? '#f87171' : maxLatency > 60 ? '#fbbf24' : '#4ade80'};font-weight:700;margin-left:6px`">
        {{ maxLatency }}ms
      </span>
    </div>

    <div v-if="data.destination">
      <span style="color:#475569;font-size:11px;text-transform:uppercase;letter-spacing:.05em">Destination</span>
      <span style="color:#38bdf8;font-weight:700;margin-left:6px">{{ data.destination.ip }}</span>
    </div>

    <div v-if="data.source">
      <span style="color:#475569;font-size:11px;text-transform:uppercase;letter-spacing:.05em">Source</span>
      <span style="color:#94a3b8;margin-left:6px">{{ data.source.country }} {{ data.source.ip }}</span>
    </div>

  </div>
</div>
</template>

<script setup>
import { computed } from 'vue'

const props = defineProps({ data: Object })

const countries = computed(() =>
  props.data ? [...new Set(props.data.hops.map(h => h.country).filter(Boolean))] : []
)

const maxLatency = computed(() => {
  if (!props.data) return 0
  const vals = props.data.hops.map(h => h.latency ?? 0)
  return vals.length ? Math.max(...vals) : 0
})
</script>
