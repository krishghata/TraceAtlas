
<template>
<div style="flex:1;display:flex;flex-direction:column;overflow:hidden;padding:12px 14px;gap:10px;box-sizing:border-box;background:#0a0f1e;color:white;font-family:'Segoe UI',sans-serif">

  <!-- ── Header ──────────────────────────────────────────────────────────── -->
  <div style="flex-shrink:0">
    <h2 style="margin:0 0 2px;font-size:15px;font-weight:700;color:#e2e8f0">Port Scanner</h2>
    <p style="margin:0;font-size:12px;color:#475569">Probe TCP ports on any host to find open services</p>
  </div>

  <!-- ── Controls ───────────────────────────────────────────────────────── -->
  <div style="display:flex;gap:8px;align-items:center;flex-shrink:0;flex-wrap:wrap">
    <input v-model="host" @keyup.enter="scan" placeholder="Host — e.g. 192.168.1.1 or example.com"
      style="flex:1;min-width:180px;padding:7px 12px;background:#0d1b2a;border:1px solid #1e3a5f;border-radius:6px;color:white;font-size:13px;outline:none"/>

    <select v-model="preset" @change="onPresetChange"
      style="padding:7px 10px;background:#0d1b2a;border:1px solid #1e3a5f;border-radius:6px;color:#94a3b8;font-size:12px">
      <option value="common">Common (22 ports)</option>
      <option value="web">Web (80,443,8080,8443,3000,4000)</option>
      <option value="database">Database (3306,5432,1433,6379,27017,9200)</option>
      <option value="remote">Remote (22,23,3389,5900,5901)</option>
      <option value="mail">Mail (25,110,143,465,587,993,995)</option>
      <option value="custom">Custom…</option>
    </select>

    <input v-if="preset === 'custom'" v-model="customPorts" placeholder="e.g. 80,443,8080-8090"
      style="width:180px;padding:7px 10px;background:#0d1b2a;border:1px solid #1e3a5f;border-radius:6px;color:white;font-size:12px;outline:none"/>

    <select v-model="timeoutMs"
      style="padding:7px 10px;background:#0d1b2a;border:1px solid #1e3a5f;border-radius:6px;color:#94a3b8;font-size:12px">
      <option :value="500">Timeout: 500ms</option>
      <option :value="1000">Timeout: 1s</option>
      <option :value="2000">Timeout: 2s</option>
      <option :value="3000">Timeout: 3s</option>
    </select>

    <button @click="scan" :disabled="scanning"
      style="padding:7px 18px;background:#1d4ed8;color:white;border:none;border-radius:6px;font-size:13px;font-weight:600;cursor:pointer">
      {{ scanning ? 'Scanning…' : 'Scan' }}
    </button>
  </div>

  <!-- ── Progress ───────────────────────────────────────────────────────── -->
  <div v-if="scanning" style="flex-shrink:0">
    <div style="display:flex;justify-content:space-between;font-size:11px;color:#475569;margin-bottom:5px">
      <span>Probing {{ activePorts.length }} ports on {{ host }}…</span>
    </div>
    <div style="height:3px;background:#1e3a5f;border-radius:2px;overflow:hidden">
      <div style="height:100%;background:#38bdf8;border-radius:2px;animation:scan-pulse 1.5s ease-in-out infinite"/>
    </div>
  </div>

  <!-- ── Error ──────────────────────────────────────────────────────────── -->
  <div v-if="error" style="padding:8px 14px;background:#450a0a;color:#fca5a5;font-size:12px;border-radius:6px;border:1px solid #7f1d1d;flex-shrink:0">
    ⚠ {{ error }}
  </div>

  <!-- ── Empty state ────────────────────────────────────────────────────── -->
  <div v-if="!scanning && !results.length && !error"
    style="flex:1;display:flex;align-items:center;justify-content:center;color:#334155;font-size:13px">
    Enter a host above and press Scan
  </div>

  <!-- ── Results ────────────────────────────────────────────────────────── -->
  <template v-if="results.length">

    <!-- Summary row -->
    <div style="display:flex;gap:18px;align-items:center;flex-shrink:0;flex-wrap:wrap">
      <div style="display:flex;gap:6px;align-items:center">
        <span style="width:10px;height:10px;border-radius:50%;background:#22c55e;display:inline-block;box-shadow:0 0 6px #22c55e"/>
        <span style="font-size:12px;font-weight:700;color:#22c55e">{{ openCount }} open</span>
      </div>
      <div style="font-size:12px;color:#475569">{{ results.length - openCount }} closed</div>
      <div style="font-size:11px;color:#334155;margin-left:auto">{{ scannedHost }}</div>
    </div>

    <!-- Port grid (visual bubbles) -->
    <div style="display:flex;flex-wrap:wrap;gap:6px;flex-shrink:0;padding:10px;background:#0d1b2a;border-radius:8px;border:1px solid #1e3a5f">
      <div v-for="r in results" :key="r.port"
        @click="selectedPort = r"
        :title="`${r.port}${r.service ? ' — ' + r.service : ''}: ${r.open ? 'OPEN' : 'closed'}`"
        :style="`padding:4px 10px;border-radius:6px;font-size:11px;font-family:monospace;cursor:pointer;transition:all 0.1s;
          ${r.open
            ? 'background:#052e16;border:1px solid #166534;color:#22c55e;box-shadow:0 0 6px rgba(34,197,94,0.3)'
            : 'background:#0a0f1e;border:1px solid #1e293b;color:#334155'}`">
        {{ r.port }}{{ r.service ? ' · ' + r.service : '' }}
      </div>
    </div>

    <!-- Detail card for selected port -->
    <div v-if="selectedPort"
      :style="`padding:12px 16px;border-radius:8px;border:1px solid ${selectedPort.open ? '#166534' : '#1e293b'};background:${selectedPort.open ? '#052e16' : '#0d1b2a'};flex-shrink:0`">
      <div style="display:flex;justify-content:space-between;align-items:center">
        <div>
          <span :style="`font-size:20px;font-weight:700;font-family:monospace;color:${selectedPort.open ? '#22c55e' : '#475569'}`">
            Port {{ selectedPort.port }}
          </span>
          <span v-if="selectedPort.service" style="font-size:13px;color:#64748b;margin-left:10px">
            {{ selectedPort.service }}
          </span>
        </div>
        <span :style="`padding:3px 12px;border-radius:12px;font-size:11px;font-weight:700;
          background:${selectedPort.open ? '#14532d' : '#1e293b'};
          color:${selectedPort.open ? '#22c55e' : '#64748b'}`">
          {{ selectedPort.open ? 'OPEN' : 'CLOSED' }}
        </span>
      </div>
      <div v-if="selectedPort.open && portInfo(selectedPort.port)" style="margin-top:8px;font-size:12px;color:#64748b;line-height:1.6">
        {{ portInfo(selectedPort.port) }}
      </div>
    </div>

    <!-- Table: open ports only -->
    <div v-if="openCount" style="flex:1;overflow-y:auto;background:#0d1b2a;border-radius:8px;border:1px solid #1e3a5f;min-height:0">
      <div style="padding:6px 14px;background:#0a1628;border-bottom:1px solid #1e3a5f;font-size:11px;font-weight:600;color:#64748b;text-transform:uppercase;letter-spacing:0.07em">
        Open Ports
      </div>
      <div v-for="(r, i) in openPorts" :key="r.port"
        :style="`display:flex;align-items:center;gap:14px;padding:8px 14px;font-size:12px;border-bottom:1px solid #0f1f35;${i%2===0?'background:#0a0f1e':'background:#0d1b2a'}`">
        <span style="font-family:monospace;font-weight:700;color:#22c55e;width:55px">{{ r.port }}</span>
        <span :style="`padding:1px 8px;border-radius:10px;font-size:10px;font-weight:700;background:#14532d;color:#22c55e;font-family:monospace;${!r.service?'opacity:0':''}`">
          {{ r.service || 'unknown' }}
        </span>
        <span style="color:#64748b;font-size:11px">{{ portInfo(r.port) }}</span>
      </div>
    </div>

  </template>

</div>
</template>

<script setup>
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const PRESETS = {
  common:   [21,22,23,25,53,80,110,139,143,443,445,587,993,995,3306,3389,5432,5900,8080,8443,27017,6379],
  web:      [80, 443, 3000, 4000, 8000, 8080, 8443, 8888, 9000],
  database: [3306, 5432, 1433, 1521, 6379, 27017, 9200, 5984, 7474],
  remote:   [22, 23, 3389, 5900, 5901, 2222, 4022],
  mail:     [25, 110, 143, 465, 587, 993, 995, 2525],
}

const PORT_INFO = {
  21:    'FTP — File Transfer Protocol',
  22:    'SSH — Secure Shell remote access',
  23:    'Telnet — Unencrypted remote access (legacy)',
  25:    'SMTP — Email sending',
  53:    'DNS — Domain name resolution',
  80:    'HTTP — Unencrypted web traffic',
  110:   'POP3 — Email retrieval',
  139:   'NetBIOS — Windows file/print sharing',
  143:   'IMAP — Email synchronisation',
  443:   'HTTPS — Encrypted web traffic',
  445:   'SMB — Windows file sharing',
  465:   'SMTPS — Encrypted email sending',
  587:   'SMTP Submission — Modern email sending',
  993:   'IMAPS — Encrypted email sync',
  995:   'POP3S — Encrypted email retrieval',
  1433:  'MSSQL — Microsoft SQL Server',
  1521:  'Oracle DB',
  2222:  'SSH (alt port)',
  3000:  'Dev server (Node/React common)',
  3306:  'MySQL / MariaDB',
  3389:  'RDP — Windows Remote Desktop',
  5432:  'PostgreSQL',
  5900:  'VNC — Remote desktop',
  5984:  'CouchDB',
  6379:  'Redis in-memory cache',
  7474:  'Neo4j browser',
  8080:  'HTTP alternate / dev server',
  8443:  'HTTPS alternate',
  8888:  'Jupyter Notebook default',
  9000:  'PHP-FPM / SonarQube',
  9200:  'Elasticsearch HTTP API',
  27017: 'MongoDB',
}

const host        = ref('')
const preset      = ref('common')
const customPorts = ref('')
const timeoutMs   = ref(1000)
const scanning    = ref(false)
const error       = ref(null)
const results     = ref([])
const scannedHost = ref('')
const selectedPort = ref(null)

function portInfo(port) { return PORT_INFO[port] ?? '' }

const activePorts = computed(() => {
  if (preset.value !== 'custom') return PRESETS[preset.value] ?? []
  // Parse "80,443,8080-8090" style
  const ports = []
  for (const part of customPorts.value.split(',')) {
    const p = part.trim()
    if (p.includes('-')) {
      const [a, b] = p.split('-').map(Number)
      if (!isNaN(a) && !isNaN(b)) for (let i = a; i <= Math.min(b, a + 256); i++) ports.push(i)
    } else {
      const n = Number(p)
      if (n > 0 && n < 65536) ports.push(n)
    }
  }
  return [...new Set(ports)].slice(0, 1024)
})

const openPorts  = computed(() => results.value.filter(r => r.open))
const openCount  = computed(() => openPorts.value.length)

function onPresetChange() { selectedPort.value = null }

async function scan() {
  const h = host.value.trim()
  if (!h) return
  const ports = activePorts.value
  if (!ports.length) { error.value = 'No ports to scan'; return }

  scanning.value     = true
  error.value        = null
  results.value      = []
  selectedPort.value = null

  try {
    const res = await invoke('scan_ports', { host: h, ports, timeoutMs: timeoutMs.value })
    results.value  = res
    scannedHost.value = h
  } catch (e) {
    error.value = String(e)
  } finally {
    scanning.value = false
  }
}
</script>

<style scoped>
@keyframes scan-pulse {
  0%, 100% { width: 20%; margin-left: 0; }
  50% { width: 60%; margin-left: 20%; }
}
</style>
