
<template>
<div style="flex:1;display:flex;flex-direction:column;overflow:hidden;padding:12px 14px;gap:10px;box-sizing:border-box;background:#0a0f1e;color:white;font-family:'Segoe UI',sans-serif">

  <!-- ── Header ──────────────────────────────────────────────────────────── -->
  <div style="flex-shrink:0">
    <h2 style="margin:0 0 2px;font-size:15px;font-weight:700;color:#e2e8f0">DNS Lookup</h2>
    <p style="margin:0;font-size:12px;color:#475569">Query DNS records for any domain across multiple resolvers</p>
  </div>

  <!-- ── Controls ───────────────────────────────────────────────────────── -->
  <div style="display:flex;gap:8px;align-items:center;flex-shrink:0;flex-wrap:wrap">
    <input v-model="domain" @keyup.enter="lookup" placeholder="Domain — e.g. google.com"
      style="flex:1;min-width:180px;padding:7px 12px;background:#0d1b2a;border:1px solid #1e3a5f;border-radius:6px;color:white;font-size:13px;outline:none"/>

    <select v-model="recordType"
      style="padding:7px 10px;background:#0d1b2a;border:1px solid #1e3a5f;border-radius:6px;color:#94a3b8;font-size:12px">
      <option v-for="t in recordTypes" :key="t" :value="t">{{ t }}</option>
    </select>

    <select v-model="dnsServer"
      style="padding:7px 10px;background:#0d1b2a;border:1px solid #1e3a5f;border-radius:6px;color:#94a3b8;font-size:12px">
      <option value="">System Default</option>
      <option value="8.8.8.8">8.8.8.8 — Google</option>
      <option value="1.1.1.1">1.1.1.1 — Cloudflare</option>
      <option value="9.9.9.9">9.9.9.9 — Quad9</option>
      <option value="208.67.222.222">208.67.222.222 — OpenDNS</option>
      <option value="custom">Custom…</option>
    </select>

    <input v-if="dnsServer === 'custom'" v-model="customServer" placeholder="e.g. 4.2.2.1"
      style="width:130px;padding:7px 10px;background:#0d1b2a;border:1px solid #1e3a5f;border-radius:6px;color:white;font-size:12px;outline:none"/>

    <button @click="lookup" :disabled="loading"
      style="padding:7px 18px;background:#1d4ed8;color:white;border:none;border-radius:6px;font-size:13px;font-weight:600;cursor:pointer">
      {{ loading ? 'Looking up…' : 'Lookup' }}
    </button>

    <!-- All-types shortcut -->
    <button @click="lookupAll" :disabled="loading"
      style="padding:7px 12px;background:#1e293b;color:#94a3b8;border:1px solid #334155;border-radius:6px;font-size:12px;cursor:pointer;white-space:nowrap">
      All Types
    </button>
  </div>

  <!-- ── Error ───────────────────────────────────────────────────────────── -->
  <div v-if="error" style="padding:8px 14px;background:#450a0a;color:#fca5a5;font-size:12px;border-radius:6px;border:1px solid #7f1d1d;flex-shrink:0">
    ⚠ {{ error }}
  </div>

  <!-- ── Results ─────────────────────────────────────────────────────────── -->
  <div style="flex:1;overflow-y:auto;display:flex;flex-direction:column;gap:10px">

    <div v-if="!results.length && !loading"
      style="flex:1;display:flex;align-items:center;justify-content:center;color:#334155;font-size:13px">
      Enter a domain above and press Lookup
    </div>

    <div v-for="res in results" :key="res.type + res.server"
      style="background:#0d1b2a;border-radius:8px;border:1px solid #1e3a5f;overflow:hidden;flex-shrink:0">

      <!-- Card header -->
      <div style="display:flex;align-items:center;gap:10px;padding:8px 14px;border-bottom:1px solid #1e3a5f;background:#0a1628">
        <span :style="`padding:2px 10px;border-radius:12px;font-size:11px;font-weight:700;background:#1e3a5f;color:${typeColor(res.type)};font-family:monospace`">
          {{ res.type }}
        </span>
        <span style="font-size:12px;font-weight:600;color:#e2e8f0">{{ res.domain }}</span>
        <span style="font-size:11px;color:#334155;margin-left:auto;font-family:monospace">
          via {{ res.server || 'system' }}
        </span>
      </div>

      <!-- Parsed records table -->
      <div v-if="res.records.length" style="padding:0">
        <div v-for="(rec, i) in res.records" :key="i"
          :style="`display:flex;gap:16px;padding:7px 14px;font-size:12px;font-family:monospace;border-bottom:1px solid #0f1f35;align-items:baseline;${i%2===0?'background:#0a0f1e':'background:#0d1b2a'}`">
          <span style="color:#475569;flex-shrink:0;width:52px">{{ rec.ttl || '' }}</span>
          <span :style="`color:${typeColor(res.type)};flex-shrink:0;width:60px`">{{ rec.rtype || res.type }}</span>
          <span style="color:#e2e8f0;word-break:break-all">{{ rec.value }}</span>
        </div>
      </div>

      <!-- Raw output fallback -->
      <pre v-else style="margin:0;padding:10px 14px;font-size:11px;color:#64748b;white-space:pre-wrap;word-break:break-all;max-height:200px;overflow-y:auto">{{ res.raw }}</pre>
    </div>

    <!-- Loading skeletons -->
    <div v-if="loading" v-for="n in pendingCount" :key="n"
      style="background:#0d1b2a;border-radius:8px;border:1px solid #1e3a5f;padding:14px;display:flex;align-items:center;gap:10px;flex-shrink:0">
      <span class="spinner"/>
      <span style="font-size:12px;color:#334155">Querying…</span>
    </div>
  </div>

</div>
</template>

<script setup>
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const recordTypes = ['A', 'AAAA', 'MX', 'NS', 'TXT', 'CNAME', 'SOA', 'PTR', 'SRV']
const domain      = ref('')
const recordType  = ref('A')
const dnsServer   = ref('')
const customServer = ref('')
const loading     = ref(false)
const error       = ref(null)
const results     = ref([])
const pendingCount = ref(0)

function effectiveServer() {
  return dnsServer.value === 'custom' ? customServer.value.trim() : dnsServer.value
}

function typeColor(type) {
  return {
    A: '#38bdf8', AAAA: '#818cf8', MX: '#f59e0b', NS: '#34d399',
    TXT: '#a78bfa', CNAME: '#fb923c', SOA: '#64748b', PTR: '#22d3ee', SRV: '#f472b6',
  }[type] ?? '#94a3b8'
}

/** Parse nslookup text output into structured records. */
function parseNslookup(raw, type) {
  const records = []
  const lines = raw.split('\n').map(l => l.trim()).filter(Boolean)

  for (const line of lines) {
    // Skip server/address header lines
    if (line.startsWith('Server:') || line.startsWith('Address:') ||
        line.startsWith('Non-authoritative') || line.startsWith('Authoritative')) continue

    // MX: "... mail exchanger = 10 smtp.google.com"
    if (type === 'MX' && line.includes('mail exchanger')) {
      const m = line.match(/mail exchanger\s*=\s*(\d+)\s+(.+)/)
      if (m) records.push({ rtype: 'MX', value: `${m[1]} ${m[2]}`, ttl: '' })
      continue
    }
    // NS: "... nameserver = ns1.google.com"
    if (type === 'NS' && line.includes('nameserver')) {
      const m = line.match(/nameserver\s*=\s*(.+)/)
      if (m) records.push({ rtype: 'NS', value: m[1].trim(), ttl: '' })
      continue
    }
    // TXT: 'text = "v=spf1 ..."'
    if (type === 'TXT' && line.includes('text =')) {
      const m = line.match(/text\s*=\s*"?(.+?)"?\s*$/)
      if (m) records.push({ rtype: 'TXT', value: m[1].trim(), ttl: '' })
      continue
    }
    // CNAME: "canonical name = ..."
    if (type === 'CNAME' && line.includes('canonical name')) {
      const m = line.match(/canonical name\s*=\s*(.+)/)
      if (m) records.push({ rtype: 'CNAME', value: m[1].trim(), ttl: '' })
      continue
    }
    // A / AAAA: lines like "Name: domain" followed by "Address: ip"
    if ((type === 'A' || type === 'AAAA') && line.startsWith('Address:')) {
      const addr = line.replace('Address:', '').trim()
      if (addr && !addr.includes('#')) records.push({ rtype: type, value: addr, ttl: '' })
      continue
    }
    // Generic fallback — include any line with meaningful content
    if (line.includes(':') && !line.startsWith('**')) {
      const [k, ...vParts] = line.split(':')
      const v = vParts.join(':').trim()
      if (v) records.push({ rtype: k.trim(), value: v, ttl: '' })
    }
  }
  return records
}

async function runLookup(type) {
  const d = domain.value.trim()
  if (!d) return null
  const server = effectiveServer()
  try {
    const res = await invoke('dns_lookup', { domain: d, recordType: type, server })
    const records = parseNslookup(res.output, type)
    return { domain: d, type, server: server || 'system', raw: res.output, records }
  } catch (e) {
    return { domain: d, type, server: server || 'system', raw: String(e), records: [] }
  }
}

async function lookup() {
  const d = domain.value.trim()
  if (!d) return
  loading.value = true
  error.value   = null
  results.value = []
  pendingCount.value = 1
  const res = await runLookup(recordType.value)
  if (res) results.value = [res]
  loading.value = false
  pendingCount.value = 0
}

async function lookupAll() {
  const d = domain.value.trim()
  if (!d) return
  loading.value = true
  error.value   = null
  results.value = []
  const types = ['A', 'AAAA', 'MX', 'NS', 'TXT', 'CNAME']
  pendingCount.value = types.length
  const all = await Promise.all(types.map(t => runLookup(t)))
  results.value = all.filter(Boolean)
  loading.value = false
  pendingCount.value = 0
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
