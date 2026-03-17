
<template>
<div style="width:270px;background:#0d1b2a;color:white;overflow-y:auto;font-family:'Segoe UI',sans-serif;font-size:13px;border-left:1px solid #1e3a5f;flex-shrink:0">

  <div style="padding:10px 14px;font-weight:700;color:#38bdf8;border-bottom:1px solid #1e3a5f;letter-spacing:.04em;font-size:12px;text-transform:uppercase">
    Insights
  </div>

  <div v-if="!insights" style="padding:16px 14px;color:#475569">
    Run a trace to see insights
  </div>

  <div v-else style="padding:10px;display:flex;flex-direction:column;gap:8px">

    <!-- Story -->
    <div style="background:#0a1628;border:1px solid #1e3a5f;border-radius:8px;padding:10px">
      <div style="color:#475569;font-size:10px;letter-spacing:.08em;text-transform:uppercase;margin-bottom:5px">Story</div>
      <div style="color:#e2e8f0;line-height:1.6">{{ insights.story }}</div>
    </div>

    <!-- Distance fun fact -->
    <div v-if="insights.distanceFact" style="background:#0a1628;border:1px solid #1e3a5f;border-radius:8px;padding:10px">
      <div style="color:#475569;font-size:10px;letter-spacing:.08em;text-transform:uppercase;margin-bottom:5px">Distance</div>
      <div style="color:#7dd3fc">🌐 {{ insights.distanceFact }}</div>
    </div>

    <!-- Country transitions -->
    <div v-if="insights.transitions?.length" style="background:#0a1628;border:1px solid #1e3a5f;border-radius:8px;padding:10px">
      <div style="color:#475569;font-size:10px;letter-spacing:.08em;text-transform:uppercase;margin-bottom:6px">Country Transitions</div>
      <div v-for="t in insights.transitions" :key="t" style="color:#e2e8f0;margin-bottom:3px">🌍 {{ t }}</div>
    </div>

    <!-- Ocean crossing -->
    <div v-if="insights.ocean" style="background:#0a1628;border:1px solid #164e63;border-radius:8px;padding:10px">
      <div style="color:#475569;font-size:10px;letter-spacing:.08em;text-transform:uppercase;margin-bottom:5px">Ocean Crossing</div>
      <div style="color:#7dd3fc">🌊 {{ insights.ocean }}</div>
    </div>

    <!-- Latency spike -->
    <div v-if="insights.latency" style="background:#0a1628;border:1px solid #7f1d1d;border-radius:8px;padding:10px">
      <div style="color:#475569;font-size:10px;letter-spacing:.08em;text-transform:uppercase;margin-bottom:5px">Latency Spike</div>
      <div style="color:#fca5a5">⚠️ {{ insights.latency }}</div>
    </div>

    <!-- Bottleneck -->
    <div v-if="insights.bottleneck" style="background:#0a1628;border:1px solid #431407;border-radius:8px;padding:10px">
      <div style="color:#475569;font-size:10px;letter-spacing:.08em;text-transform:uppercase;margin-bottom:5px">Bottleneck</div>
      <div style="color:#fdba74">🐢 {{ insights.bottleneck }}</div>
    </div>

  </div>
</div>
</template>

<script setup>
defineProps({ insights: Object })
</script>
