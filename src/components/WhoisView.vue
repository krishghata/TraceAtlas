
<template>
<div style="flex:1;display:flex;flex-direction:column;overflow:hidden;padding:12px 14px;gap:10px;box-sizing:border-box;background:#0a0f1e;color:white;font-family:'Segoe UI',sans-serif">

  <!-- ── Header ──────────────────────────────────────────────────────────── -->
  <div style="flex-shrink:0">
    <h2 style="margin:0 0 2px;font-size:15px;font-weight:700;color:#e2e8f0">Whois Lookup</h2>
    <p style="margin:0;font-size:12px;color:#475569">Domain registration info and IP block ownership</p>
  </div>

  <!-- ── Controls ───────────────────────────────────────────────────────── -->
  <div style="display:flex;gap:8px;align-items:center;flex-shrink:0">
    <input v-model="query" @keyup.enter="lookup" placeholder="Domain or IP — e.g. github.com or 8.8.8.8"
      style="flex:1;padding:7px 12px;background:#0d1b2a;border:1px solid #1e3a5f;border-radius:6px;color:white;font-size:13px;outline:none"/>
    <button @click="lookup" :disabled="loading"
      style="padding:7px 18px;background:#1d4ed8;color:white;border:none;border-radius:6px;font-size:13px;font-weight:600;cursor:pointer">
      {{ loading ? 'Looking up…' : 'Lookup' }}
    </button>
  </div>

  <!-- ── Error ──────────────────────────────────────────────────────────── -->
  <div v-if="error" style="padding:8px 14px;background:#450a0a;color:#fca5a5;font-size:12px;border-radius:6px;border:1px solid #7f1d1d;flex-shrink:0">
    ⚠ {{ error }}
  </div>

  <!-- ── Loading ────────────────────────────────────────────────────────── -->
  <div v-if="loading" style="display:flex;align-items:center;gap:10px;flex-shrink:0;color:#334155;font-size:12px">
    <span class="spinner"/> Querying WHOIS servers…
  </div>

  <!-- ── Empty state ────────────────────────────────────────────────────── -->
  <div v-if="!loading && !sections.length && !error"
    style="flex:1;display:flex;align-items:center;justify-content:center;color:#334155;font-size:13px">
    Enter a domain or IP above and press Lookup
  </div>

  <!-- ── Results ────────────────────────────────────────────────────────── -->
  <div v-if="sections.length" style="flex:1;overflow-y:auto;display:flex;flex-direction:column;gap:10px">

    <!-- Highlight cards for key fields -->
    <div style="display:grid;grid-template-columns:repeat(auto-fill,minmax(200px,1fr));gap:8px;flex-shrink:0">
      <div v-for="h in highlights" :key="h.label"
        :style="`padding:10px 14px;border-radius:8px;border:1px solid ${h.urgent?'#7f1d1d':'#1e3a5f'};background:${h.urgent?'#1a0000':'#0d1b2a'}`">
        <div style="font-size:10px;text-transform:uppercase;letter-spacing:0.07em;color:#475569;margin-bottom:4px">{{ h.label }}</div>
        <div :style="`font-size:12px;font-weight:600;color:${h.color};font-family:monospace;word-break:break-all`">{{ h.value }}</div>
      </div>
    </div>

    <!-- Full sections -->
    <div v-for="sec in sections" :key="sec.title"
      style="background:#0d1b2a;border-radius:8px;border:1px solid #1e3a5f;overflow:hidden;flex-shrink:0">
      <div style="padding:7px 14px;background:#0a1628;border-bottom:1px solid #1e3a5f;font-size:11px;font-weight:600;color:#64748b;text-transform:uppercase;letter-spacing:0.07em">
        {{ sec.title }}
      </div>
      <div v-for="(row, i) in sec.rows" :key="i"
        :style="`display:flex;gap:0;font-size:12px;border-bottom:1px solid #0f1f35;${i%2===0?'background:#0a0f1e':'background:#0d1b2a'}`">
        <span style="padding:6px 14px;color:#475569;font-family:monospace;flex-shrink:0;width:200px;word-break:break-word">{{ row.key }}</span>
        <span style="padding:6px 14px;color:#e2e8f0;font-family:monospace;flex:1;word-break:break-all">{{ row.value }}</span>
      </div>
    </div>

    <!-- Raw toggle -->
    <div style="flex-shrink:0">
      <button @click="showRaw = !showRaw"
        style="padding:5px 12px;background:#1e293b;color:#64748b;border:1px solid #334155;border-radius:6px;font-size:11px;cursor:pointer">
        {{ showRaw ? 'Hide raw' : 'Show raw output' }}
      </button>
      <pre v-if="showRaw" style="margin-top:8px;padding:12px;background:#0d1b2a;border-radius:6px;border:1px solid #1e3a5f;font-size:11px;color:#64748b;white-space:pre-wrap;word-break:break-all;max-height:300px;overflow-y:auto">{{ rawOutput }}</pre>
    </div>
  </div>

</div>
</template>

<script setup>
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const query     = ref('')
const loading   = ref(false)
const error     = ref(null)
const rawOutput = ref('')
const showRaw   = ref(false)
const sections  = ref([])

// ── Parse raw WHOIS output into sections ─────────────────────────────────────
function parseWhois(raw) {
  const secs = []
  let   cur  = { title: 'General', rows: [] }
  secs.push(cur)

  const sectionHeaders = [
    'registrant', 'admin', 'tech', 'billing', 'registrar',
    'name server', 'dnssec', 'network', 'contact', 'organisation',
  ]

  for (const line of raw.split('\n')) {
    const trimmed = line.trim()
    if (!trimmed || trimmed.startsWith('%') || trimmed.startsWith('#') ||
        trimmed.startsWith('--') || trimmed.startsWith('>>')) continue

    const colonIdx = trimmed.indexOf(':')
    if (colonIdx === -1) continue

    const key   = trimmed.slice(0, colonIdx).trim()
    const value = trimmed.slice(colonIdx + 1).trim()
    if (!value) continue

    // Detect section breaks
    const lkey = key.toLowerCase()
    const isHeader = sectionHeaders.some(h => lkey === h || lkey.startsWith(h + ' '))
    if (isHeader && cur.rows.length > 0) {
      cur = { title: key, rows: [] }
      secs.push(cur)
    }

    cur.rows.push({ key, value })
  }

  return secs.filter(s => s.rows.length > 0)
}

// ── Extract highlight fields ──────────────────────────────────────────────────
const HIGHLIGHT_KEYS = [
  { labels: ['domain name', 'domain'],                   out: 'Domain',      color: '#38bdf8' },
  { labels: ['registrar'],                                out: 'Registrar',   color: '#94a3b8' },
  { labels: ['creation date', 'created', 'registered'],  out: 'Registered',  color: '#94a3b8' },
  { labels: ['registry expiry date', 'expiry date', 'expires', 'expiration date'], out: 'Expires', color: '#f59e0b', urgent: true },
  { labels: ['updated date', 'last updated', 'changed'], out: 'Last Updated', color: '#94a3b8' },
  { labels: ['registrant organization', 'org-name', 'org', 'organization'], out: 'Organization', color: '#34d399' },
  { labels: ['registrant country', 'country'],           out: 'Country',     color: '#94a3b8' },
]

const highlights = computed(() => {
  const allRows = sections.value.flatMap(s => s.rows)
  return HIGHLIGHT_KEYS.flatMap(({ labels, out, color, urgent }) => {
    const row = allRows.find(r => labels.some(l => r.key.toLowerCase().includes(l)))
    if (!row) return []
    // Check if expiry is soon (within 60 days)
    let isUrgent = false
    if (urgent && row.value) {
      const d = new Date(row.value)
      if (!isNaN(d) && (d - Date.now()) < 60 * 24 * 3600 * 1000) isUrgent = true
    }
    return [{ label: out, value: row.value, color: isUrgent ? '#ef4444' : color, urgent: isUrgent }]
  })
})

async function lookup() {
  const q = query.value.trim()
  if (!q) return
  loading.value  = true
  error.value    = null
  sections.value = []
  rawOutput.value = ''
  showRaw.value  = false
  try {
    const raw = await invoke('whois_lookup', { query: q })
    rawOutput.value = raw
    sections.value  = parseWhois(raw)
    if (!sections.value.length) {
      sections.value = [{ title: 'Output', rows: [{ key: 'raw', value: raw }] }]
    }
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
