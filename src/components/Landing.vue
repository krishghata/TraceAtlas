
<template>
<div style="display:flex;flex-direction:column;align-items:center;justify-content:center;min-height:100vh;background:#0a0f1e;color:white;font-family:'Segoe UI',sans-serif;text-align:center;padding:40px">

  <img src="/logo-light.svg" alt="TraceAtlas" style="width:130px;height:130px;margin-bottom:22px;filter:drop-shadow(0 0 28px rgba(56,189,248,0.5))"/>

  <h1 style="font-size:32px;font-weight:700;color:#e2e8f0;margin:0 0 8px;letter-spacing:-0.5px">TraceAtlas</h1>
  <p style="font-size:14px;color:#64748b;margin:0 0 28px;max-width:340px;line-height:1.6">
    Visualize how your internet traffic travels — hop by hop, country by country, across the globe.
  </p>

  <!-- Progress bar -->
  <div style="width:180px;height:3px;background:#1e3a5f;border-radius:2px;overflow:hidden;margin-bottom:10px">
    <div :style="`height:100%;background:#38bdf8;border-radius:2px;transition:width 0.1s linear;width:${progress}%`"></div>
  </div>
  <p style="font-size:11px;color:#334155;margin:0">Launching in {{ countdown }}s…</p>

  <p style="margin-top:40px;font-size:11px;color:#1e3a5f">Tauri · Vue 3 · Leaflet · ip-api.com</p>
</div>
</template>

<script setup>
import { ref, onMounted, onUnmounted } from 'vue'

const emit     = defineEmits(['start'])
const countdown = ref(3)
const progress  = ref(0)

let timer = null
const TOTAL_MS = 3000
const TICK_MS  = 50

onMounted(() => {
  let elapsed = 0
  timer = setInterval(() => {
    elapsed += TICK_MS
    progress.value  = Math.min((elapsed / TOTAL_MS) * 100, 100)
    countdown.value = Math.max(Math.ceil((TOTAL_MS - elapsed) / 1000), 0)
    if (elapsed >= TOTAL_MS) {
      clearInterval(timer)
      emit('start')
    }
  }, TICK_MS)
})

onUnmounted(() => { if (timer) clearInterval(timer) })
</script>
