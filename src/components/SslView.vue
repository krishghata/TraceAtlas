
<template>
<div style="flex:1;display:flex;flex-direction:column;overflow:hidden;padding:12px 14px;gap:10px;box-sizing:border-box;background:#0a0f1e;color:white;font-family:'Segoe UI',sans-serif">

  <!-- ── Header ──────────────────────────────────────────────────────────── -->
  <div style="flex-shrink:0">
    <h2 style="margin:0 0 2px;font-size:15px;font-weight:700;color:#e2e8f0">SSL / TLS Inspector</h2>
    <p style="margin:0;font-size:12px;color:#475569">Certificate validity, expiry, SANs, cipher suite and chain info</p>
  </div>

  <!-- ── Controls ───────────────────────────────────────────────────────── -->
  <div style="display:flex;gap:8px;align-items:center;flex-shrink:0;flex-wrap:wrap">
    <input v-model="host" @keyup.enter="inspect" placeholder="Host — e.g. github.com or 192.168.1.1"
      style="flex:1;min-width:200px;padding:7px 12px;background:#0d1b2a;border:1px solid #1e3a5f;border-radius:6px;color:white;font-size:13px;outline:none"
      :disabled="loading"/>
    <input v-model.number="port" placeholder="443"
      style="width:70px;padding:7px 10px;background:#0d1b2a;border:1px solid #1e3a5f;border-radius:6px;color:#94a3b8;font-size:12px;outline:none"
      :disabled="loading"/>
    <button @click="inspect" :disabled="loading"
      style="padding:7px 18px;background:#1d4ed8;color:white;border:none;border-radius:6px;font-size:13px;font-weight:600;cursor:pointer">
      {{ loading ? 'Connecting…' : '🔒 Inspect' }}
    </button>
  </div>

  <!-- ── Error ──────────────────────────────────────────────────────────── -->
  <div v-if="error" style="padding:8px 14px;background:#450a0a;color:#fca5a5;font-size:12px;border-radius:6px;border:1px solid #7f1d1d;flex-shrink:0">
    ⚠ {{ error }}
  </div>

  <!-- ── Loading ────────────────────────────────────────────────────────── -->
  <div v-if="loading" style="flex-shrink:0;font-size:12px;color:#475569;display:flex;align-items:center;gap:8px">
    <span class="spinner"/> Connecting to {{ host }}:{{ port || 443 }}…
  </div>

  <!-- ── Empty state ────────────────────────────────────────────────────── -->
  <div v-if="!loading && !cert && !error"
    style="flex:1;display:flex;align-items:center;justify-content:center;color:#334155;font-size:13px">
    Enter a hostname above and press Inspect
  </div>

  <!-- ── Results ────────────────────────────────────────────────────────── -->
  <div v-if="cert" style="flex:1;overflow-y:auto;display:flex;flex-direction:column;gap:10px">

    <!-- Expiry banner -->
    <div :style="`padding:12px 16px;border-radius:8px;border:1px solid ${expiryBorderColor};background:${expiryBgColor};flex-shrink:0;display:flex;align-items:center;gap:14px`">
      <span style="font-size:28px">{{ expiryIcon }}</span>
      <div>
        <div :style="`font-size:14px;font-weight:700;color:${expiryTextColor}`">
          {{ expiryLabel }}
        </div>
        <div style="font-size:12px;color:#64748b;margin-top:2px">
          Valid until {{ cert.not_after }}
        </div>
      </div>
      <div style="margin-left:auto;text-align:right">
        <div :style="`font-size:26px;font-weight:800;font-family:monospace;color:${expiryTextColor}`">
          {{ cert.days_left }}
        </div>
        <div style="font-size:10px;color:#475569">days left</div>
      </div>
    </div>

    <!-- Key info cards -->
    <div style="display:grid;grid-template-columns:repeat(auto-fill,minmax(200px,1fr));gap:8px;flex-shrink:0">
      <div v-for="card in infoCards" :key="card.label"
        style="padding:10px 14px;border-radius:8px;border:1px solid #1e3a5f;background:#0d1b2a">
        <div style="font-size:10px;text-transform:uppercase;letter-spacing:0.07em;color:#475569;margin-bottom:4px">{{ card.label }}</div>
        <div style="font-size:12px;font-weight:600;color:#e2e8f0;font-family:monospace;word-break:break-all">{{ card.value || '—' }}</div>
      </div>
    </div>

    <!-- SANs -->
    <div v-if="cert.sans.length" style="flex-shrink:0;background:#0d1b2a;border-radius:8px;border:1px solid #1e3a5f;overflow:hidden">
      <div style="padding:7px 14px;background:#0a1628;border-bottom:1px solid #1e3a5f;font-size:11px;font-weight:600;color:#64748b;text-transform:uppercase;letter-spacing:0.07em">
        Subject Alternative Names ({{ cert.sans.length }})
      </div>
      <div style="padding:10px 14px;display:flex;flex-wrap:wrap;gap:6px">
        <span v-for="san in cert.sans" :key="san"
          style="padding:2px 10px;border-radius:10px;font-size:11px;font-family:monospace;background:#0a1628;border:1px solid #1e3a5f;color:#38bdf8">
          {{ san }}
        </span>
      </div>
    </div>

    <!-- Fingerprint & serial -->
    <div style="flex-shrink:0;background:#0d1b2a;border-radius:8px;border:1px solid #1e3a5f;overflow:hidden">
      <div style="padding:7px 14px;background:#0a1628;border-bottom:1px solid #1e3a5f;font-size:11px;font-weight:600;color:#64748b;text-transform:uppercase;letter-spacing:0.07em">
        Certificate Details
      </div>
      <div v-for="(row, i) in detailRows" :key="row.k"
        :style="`display:flex;gap:0;font-size:12px;border-bottom:1px solid #0f1f35;${i%2===0?'background:#0a0f1e':'background:#0d1b2a'}`">
        <span style="padding:7px 14px;color:#475569;font-family:monospace;flex-shrink:0;width:160px">{{ row.k }}</span>
        <span style="padding:7px 14px;color:#e2e8f0;font-family:monospace;flex:1;word-break:break-all">{{ row.v || '—' }}</span>
      </div>
    </div>

  </div>

</div>
</template>

<script setup>
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const host    = ref('')
const port    = ref(443)
const loading = ref(false)
const error   = ref(null)
const cert    = ref(null)

// ── Expiry styling ────────────────────────────────────────────────────────────
const expiryIcon      = computed(() => {
  if (!cert.value) return ''
  const d = cert.value.days_left
  return d < 0 ? '🔴' : d < 14 ? '🟠' : d < 30 ? '🟡' : '🟢'
})
const expiryLabel     = computed(() => {
  if (!cert.value) return ''
  const d = cert.value.days_left
  return d < 0 ? 'Certificate EXPIRED' : d < 14 ? 'Expiring very soon!' : d < 30 ? 'Expiring soon' : 'Certificate valid'
})
const expiryTextColor = computed(() => {
  if (!cert.value) return '#e2e8f0'
  const d = cert.value.days_left
  return d < 0 ? '#ef4444' : d < 14 ? '#f97316' : d < 30 ? '#f59e0b' : '#22c55e'
})
const expiryBorderColor = computed(() => {
  if (!cert.value) return '#1e3a5f'
  const d = cert.value.days_left
  return d < 0 ? '#7f1d1d' : d < 14 ? '#7c2d12' : d < 30 ? '#78350f' : '#166534'
})
const expiryBgColor = computed(() => {
  if (!cert.value) return '#0d1b2a'
  const d = cert.value.days_left
  return d < 0 ? '#1a0000' : d < 14 ? '#1c0e00' : d < 30 ? '#1a1000' : '#052e16'
})

// ── Info cards ────────────────────────────────────────────────────────────────
const infoCards = computed(() => {
  if (!cert.value) return []
  return [
    { label: 'Subject',  value: cert.value.subject  },
    { label: 'Issuer',   value: cert.value.issuer   },
    { label: 'Protocol', value: cert.value.protocol },
    { label: 'Cipher',   value: cert.value.cipher   },
    { label: 'Valid From', value: cert.value.not_before },
    { label: 'Valid Until', value: cert.value.not_after },
  ]
})

const detailRows = computed(() => {
  if (!cert.value) return []
  return [
    { k: 'Serial Number', v: cert.value.serial },
    { k: 'SHA-256 Fingerprint', v: cert.value.fingerprint },
  ]
})

async function inspect() {
  const h = host.value.trim()
  if (!h) return
  loading.value = true
  error.value   = null
  cert.value    = null
  try {
    cert.value = await invoke('ssl_inspect', { host: h, port: port.value || 443 })
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
