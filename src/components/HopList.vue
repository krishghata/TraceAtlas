
<template>
<div style="width:220px;background:#1e293b;color:white;overflow-y:auto;font-family:sans-serif;font-size:13px">
  <div style="padding:10px 12px;font-weight:bold;color:#38bdf8;border-bottom:1px solid #334155">
    Hops {{ hops ? `(${hops.length})` : '' }}
  </div>
  <div v-if="!hops" style="padding:12px;color:#64748b">No trace yet</div>
  <div
    v-for="h in hops"
    :key="h.hop"
    style="padding:8px 12px;border-bottom:1px solid #1e293b"
  >
    <div style="display:flex;justify-content:space-between;margin-bottom:2px">
      <span style="color:#38bdf8;font-weight:bold">#{{ h.hop }}</span>
      <span style="color:#94a3b8">{{ h.latency != null ? h.latency + 'ms' : '—' }}</span>
    </div>
    <div style="color:#e2e8f0;margin-bottom:1px">{{ h.country || '?' }}</div>
    <div style="color:#64748b;font-size:11px;word-break:break-all">{{ h.ip }}</div>
    <div style="color:#94a3b8;font-size:11px;margin-top:1px">{{ h.org || '' }}</div>
  </div>
</div>
</template>

<script setup>
defineProps({ hops: Array })
</script>
