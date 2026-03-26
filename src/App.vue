
<template>
<div v-if="page === 'home'">
  <Landing @start="page = 'app'" />
</div>

<div v-else
  style="height:100vh;display:flex;flex-direction:column;overflow:hidden;background:#0a0f1e;color:white;font-family:'Segoe UI',sans-serif">

  <!-- ── Tab bar ─────────────────────────────────────────────────────────── -->
  <div style="display:flex;align-items:stretch;background:#0d1b2a;border-bottom:1px solid #1e3a5f;flex-shrink:0;padding:0 6px">
    <button v-for="tab in tabs" :key="tab.id" @click="activeTab = tab.id"
      :style="`padding:9px 18px;background:none;border:none;border-bottom:2px solid ${activeTab===tab.id ? '#38bdf8' : 'transparent'};color:${activeTab===tab.id ? '#38bdf8' : '#64748b'};font-size:12px;font-weight:600;cursor:pointer;font-family:'Segoe UI',sans-serif;white-space:nowrap;transition:color 0.15s,border-color 0.15s`">
      {{ tab.label }}
    </button>
  </div>

  <!-- ── Active view fills remaining space ──────────────────────────────── -->
  <AppView          v-if="activeTab === 'trace'"  />
  <PingView         v-else-if="activeTab === 'ping'"   />
  <NetworkHealthView v-else-if="activeTab === 'health'" />

</div>
</template>

<script setup>
import { ref } from 'vue'
import Landing           from './components/Landing.vue'
import AppView           from './components/AppView.vue'
import PingView          from './components/PingView.vue'
import NetworkHealthView from './components/NetworkHealthView.vue'

const page      = ref('home')
const activeTab = ref('trace')

const tabs = [
  { id: 'trace',  label: '⤷ Traceroute'     },
  { id: 'ping',   label: '📡 Ping'           },
  { id: 'health', label: '🩺 Network Health' },
]
</script>
