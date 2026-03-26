
<template>
<div style="width:200px;flex-shrink:0;background:#0d1b2a;color:white;display:flex;flex-direction:column;overflow:hidden;border-right:1px solid #1e3a5f;font-family:'Segoe UI',sans-serif">

  <!-- Header -->
  <div style="padding:7px 10px;font-size:11px;font-weight:700;color:#38bdf8;border-bottom:1px solid #1e3a5f;flex-shrink:0;text-transform:uppercase;letter-spacing:0.07em">
    Hops {{ hops ? `(${hops.length})` : '' }}
  </div>

  <div v-if="!hops" style="padding:12px;color:#334155;font-size:12px">No trace yet</div>

  <!-- Hop rows — fill remaining height, no scroll -->
  <div style="flex:1;display:flex;flex-direction:column;overflow:hidden">
    <div
      v-for="h in hops"
      :key="h.hop"
      :style="`padding:${itemPad};border-bottom:1px solid #0f1f35;flex:1;display:flex;flex-direction:column;justify-content:center;min-height:0`"
    >
      <div style="display:flex;justify-content:space-between;align-items:center">
        <span :style="`color:#38bdf8;font-weight:700;font-size:${fs.hop}`">#{{ h.hop }}</span>
        <span :style="`color:${latColor(h.latency)};font-size:${fs.latency};font-family:monospace`">
          {{ h.latency != null ? h.latency + 'ms' : '—' }}
        </span>
      </div>
      <div :style="`color:#e2e8f0;font-size:${fs.country};margin-top:1px`">{{ h.country || '?' }}</div>
      <div :style="`color:#475569;font-size:${fs.ip};font-family:monospace;overflow:hidden;text-overflow:ellipsis;white-space:nowrap`">{{ h.ip }}</div>
      <div v-if="fs.showOrg" :style="`color:#334155;font-size:${fs.org};overflow:hidden;text-overflow:ellipsis;white-space:nowrap`">{{ h.org || '' }}</div>
    </div>
  </div>

</div>
</template>

<script setup>
import { computed } from 'vue'

const props = defineProps({ hops: Array })

function latColor(ms) {
  if (ms == null) return '#334155'
  return ms < 50 ? '#22c55e' : ms < 150 ? '#f59e0b' : ms < 300 ? '#f97316' : '#ef4444'
}

// Auto-scale font sizes and padding based on hop count so all hops fit
const fs = computed(() => {
  const n = props.hops?.length ?? 0
  if (n <= 6)  return { hop: '12px', latency: '11px', country: '11px', ip: '10px', org: '10px', showOrg: true,  pad: '7px 10px' }
  if (n <= 10) return { hop: '11px', latency: '10px', country: '10px', ip: '9px',  org: '9px',  showOrg: true,  pad: '4px 10px' }
  if (n <= 15) return { hop: '10px', latency: '10px', country: '10px', ip: '9px',  org: '9px',  showOrg: false, pad: '3px 10px' }
  return             { hop: '10px', latency: '9px',  country: '9px',  ip: '9px',  org: '9px',  showOrg: false, pad: '2px 8px'  }
})

const itemPad = computed(() => fs.value.pad)
</script>
