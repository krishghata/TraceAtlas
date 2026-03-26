
<template>
<div style="flex:1;display:flex;flex-direction:column;overflow:hidden;padding:12px 14px;gap:10px;box-sizing:border-box;background:#0a0f1e;color:white;font-family:'Segoe UI',sans-serif">

  <!-- ── Header ──────────────────────────────────────────────────────────── -->
  <div style="flex-shrink:0">
    <h2 style="margin:0 0 2px;font-size:15px;font-weight:700;color:#e2e8f0">HTTP Header Inspector</h2>
    <p style="margin:0;font-size:12px;color:#475569">Response headers, redirect chain, timing, and security score</p>
  </div>

  <!-- ── Controls ───────────────────────────────────────────────────────── -->
  <div style="display:flex;gap:8px;align-items:center;flex-shrink:0;flex-wrap:wrap">
    <input v-model="url" @keyup.enter="run" placeholder="URL — e.g. https://example.com"
      style="flex:1;min-width:240px;padding:7px 12px;background:#0d1b2a;border:1px solid #1e3a5f;border-radius:6px;color:white;font-size:13px;outline:none"
      :disabled="loading"/>
    <button @click="run" :disabled="loading"
      style="padding:7px 18px;background:#1d4ed8;color:white;border:none;border-radius:6px;font-size:13px;font-weight:600;cursor:pointer">
      {{ loading ? 'Fetching…' : '🌐 Inspect' }}
    </button>
  </div>

  <!-- ── Error ──────────────────────────────────────────────────────────── -->
  <div v-if="error" style="padding:8px 14px;background:#450a0a;color:#fca5a5;font-size:12px;border-radius:6px;border:1px solid #7f1d1d;flex-shrink:0">
    ⚠ {{ error }}
    <div style="margin-top:4px;font-size:11px;color:#7f1d1d">Make sure <code>curl</code> is available (included in Windows 10+ and macOS).</div>
  </div>

  <!-- ── Loading ────────────────────────────────────────────────────────── -->
  <div v-if="loading" style="flex-shrink:0;font-size:12px;color:#475569;display:flex;align-items:center;gap:8px">
    <span class="spinner"/> Sending request…
  </div>

  <!-- ── Empty state ────────────────────────────────────────────────────── -->
  <div v-if="!loading && !result && !error"
    style="flex:1;display:flex;align-items:center;justify-content:center;color:#334155;font-size:13px">
    Enter a URL above and press Inspect
  </div>

  <!-- ── Results ────────────────────────────────────────────────────────── -->
  <div v-if="result" style="flex:1;overflow-y:auto;display:flex;flex-direction:column;gap:10px">

    <!-- Status + timing + security score -->
    <div style="display:grid;grid-template-columns:repeat(auto-fill,minmax(160px,1fr));gap:8px;flex-shrink:0">
      <!-- Status -->
      <div :style="`padding:10px 14px;border-radius:8px;border:1px solid ${statusBorder};background:${statusBg}`">
        <div style="font-size:10px;text-transform:uppercase;letter-spacing:0.07em;color:#475569;margin-bottom:4px">Status</div>
        <div :style="`font-size:18px;font-weight:800;font-family:monospace;color:${statusColor}`">
          {{ result.status }}
        </div>
        <div style="font-size:11px;color:#64748b">{{ result.status_text }}</div>
      </div>

      <!-- Protocol -->
      <div style="padding:10px 14px;border-radius:8px;border:1px solid #1e3a5f;background:#0d1b2a">
        <div style="font-size:10px;text-transform:uppercase;letter-spacing:0.07em;color:#475569;margin-bottom:4px">Protocol</div>
        <div style="font-size:14px;font-weight:700;color:#38bdf8;font-family:monospace">{{ result.protocol || '—' }}</div>
      </div>

      <!-- Timing -->
      <div style="padding:10px 14px;border-radius:8px;border:1px solid #1e3a5f;background:#0d1b2a">
        <div style="font-size:10px;text-transform:uppercase;letter-spacing:0.07em;color:#475569;margin-bottom:4px">Timing</div>
        <div :style="`font-size:14px;font-weight:700;font-family:monospace;color:${timingColor}`">{{ result.timing_ms }}ms</div>
      </div>

      <!-- Security score -->
      <div :style="`padding:10px 14px;border-radius:8px;border:1px solid ${scoreBorder};background:${scoreBg}`">
        <div style="font-size:10px;text-transform:uppercase;letter-spacing:0.07em;color:#475569;margin-bottom:4px">Security Score</div>
        <div :style="`font-size:22px;font-weight:800;font-family:monospace;color:${scoreColor}`">
          {{ result.security_score }}<span style="font-size:12px">/100</span>
        </div>
      </div>
    </div>

    <!-- Security notes -->
    <div v-if="result.security_notes.length"
      style="flex-shrink:0;padding:10px 14px;background:#1a0f00;border:1px solid #78350f;border-radius:8px">
      <div style="font-size:11px;font-weight:700;color:#f59e0b;text-transform:uppercase;letter-spacing:0.07em;margin-bottom:8px">
        Security Findings
      </div>
      <div v-for="note in result.security_notes" :key="note"
        style="display:flex;gap:8px;align-items:flex-start;margin-bottom:5px;font-size:12px;color:#fbbf24">
        <span style="flex-shrink:0">⚠</span>
        <span>{{ note }}</span>
      </div>
    </div>

    <!-- Redirect chain -->
    <div v-if="result.redirects.length"
      style="flex-shrink:0;background:#0d1b2a;border-radius:8px;border:1px solid #1e3a5f;overflow:hidden">
      <div style="padding:7px 14px;background:#0a1628;border-bottom:1px solid #1e3a5f;font-size:11px;font-weight:600;color:#64748b;text-transform:uppercase;letter-spacing:0.07em">
        Redirect Chain ({{ result.redirects.length }})
      </div>
      <div v-for="(r, i) in result.redirects" :key="i"
        :style="`display:flex;gap:10px;align-items:center;padding:7px 14px;font-size:12px;border-bottom:1px solid #0f1f35;${i%2===0?'background:#0a0f1e':'background:#0d1b2a'}`">
        <span :style="`padding:1px 8px;border-radius:4px;font-size:11px;font-weight:700;font-family:monospace;
          background:${r.status<400?'#1e3a5f':'#450a0a'};color:${r.status<400?'#38bdf8':'#fca5a5'}`">
          {{ r.status }}
        </span>
        <span style="color:#94a3b8;font-family:monospace;word-break:break-all">{{ r.url }}</span>
      </div>
    </div>

    <!-- Headers table -->
    <div style="flex:1;overflow-y:auto;background:#0d1b2a;border-radius:8px;border:1px solid #1e3a5f;min-height:0">
      <div style="padding:7px 14px;background:#0a1628;border-bottom:1px solid #1e3a5f;font-size:11px;font-weight:600;color:#64748b;text-transform:uppercase;letter-spacing:0.07em">
        Response Headers ({{ result.headers.length }})
      </div>
      <div v-for="([k, v], i) in result.headers" :key="k"
        :style="`display:flex;gap:0;font-size:12px;border-bottom:1px solid #0f1f35;${i%2===0?'background:#0a0f1e':'background:#0d1b2a'}`">
        <span :style="`padding:6px 14px;font-family:monospace;flex-shrink:0;width:220px;word-break:break-word;color:${headerColor(k)}`">
          {{ k }}
        </span>
        <span style="padding:6px 14px;color:#e2e8f0;font-family:monospace;flex:1;word-break:break-all">{{ v }}</span>
      </div>
    </div>

  </div>

</div>
</template>

<script setup>
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const url     = ref('')
const loading = ref(false)
const error   = ref(null)
const result  = ref(null)

// ── Colors ────────────────────────────────────────────────────────────────────
const statusColor  = computed(() => {
  const s = result.value?.status ?? 0
  return s < 300 ? '#22c55e' : s < 400 ? '#f59e0b' : s < 500 ? '#f97316' : '#ef4444'
})
const statusBorder = computed(() => {
  const s = result.value?.status ?? 0
  return s < 300 ? '#166534' : s < 400 ? '#78350f' : '#7f1d1d'
})
const statusBg     = computed(() => {
  const s = result.value?.status ?? 0
  return s < 300 ? '#052e16' : s < 400 ? '#1c0e00' : '#1a0000'
})
const timingColor  = computed(() => {
  const t = result.value?.timing_ms ?? 0
  return t < 300 ? '#22c55e' : t < 1000 ? '#f59e0b' : '#ef4444'
})
const scoreColor   = computed(() => {
  const s = result.value?.security_score ?? 0
  return s >= 80 ? '#22c55e' : s >= 60 ? '#f59e0b' : '#ef4444'
})
const scoreBorder  = computed(() => {
  const s = result.value?.security_score ?? 0
  return s >= 80 ? '#166534' : s >= 60 ? '#78350f' : '#7f1d1d'
})
const scoreBg      = computed(() => {
  const s = result.value?.security_score ?? 0
  return s >= 80 ? '#052e16' : s >= 60 ? '#1c0e00' : '#1a0000'
})

const SECURITY_HEADERS = new Set([
  'strict-transport-security','content-security-policy','x-frame-options',
  'x-content-type-options','referrer-policy','permissions-policy',
])
function headerColor(k) {
  const lk = k.toLowerCase()
  if (SECURITY_HEADERS.has(lk)) return '#22c55e'
  if (lk === 'x-powered-by' || lk === 'server') return '#f59e0b'
  if (lk.startsWith('content-')) return '#38bdf8'
  return '#94a3b8'
}

async function run() {
  let u = url.value.trim()
  if (!u) return
  if (!u.startsWith('http://') && !u.startsWith('https://')) u = 'https://' + u
  url.value     = u
  loading.value = true
  error.value   = null
  result.value  = null
  try {
    result.value = await invoke('http_inspect', { url: u })
  } catch (e) {
    error.value = String(e)
  } finally {
    loading.value = false
  }
}
</script>

<style scoped>
.spinner {
  display: inline-block;
  width: 12px; height: 12px;
  border: 2px solid #1e3a5f;
  border-top-color: #38bdf8;
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
  flex-shrink: 0;
}
@keyframes spin { to { transform: rotate(360deg); } }
</style>
