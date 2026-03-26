
<template>
<div style="flex:1;display:flex;flex-direction:column;overflow:hidden;padding:12px 14px;gap:10px;box-sizing:border-box;background:#0a0f1e;color:white;font-family:'Segoe UI',sans-serif">

  <!-- ── Header ──────────────────────────────────────────────────────────── -->
  <div style="flex-shrink:0">
    <h2 style="margin:0 0 2px;font-size:15px;font-weight:700;color:#e2e8f0">Wake on LAN</h2>
    <p style="margin:0;font-size:12px;color:#475569">Send a magic packet to wake up a sleeping device on your local network</p>
  </div>

  <!-- ── Controls ───────────────────────────────────────────────────────── -->
  <div style="display:flex;gap:8px;align-items:center;flex-shrink:0;flex-wrap:wrap">
    <input v-model="mac" @keyup.enter="wake" placeholder="MAC address — e.g. AA:BB:CC:DD:EE:FF"
      style="flex:1;min-width:240px;padding:7px 12px;background:#0d1b2a;border:1px solid #1e3a5f;border-radius:6px;color:white;font-size:13px;outline:none;font-family:monospace"
      :disabled="sending"/>
    <input v-model="label" placeholder="Label (optional)"
      style="width:160px;padding:7px 10px;background:#0d1b2a;border:1px solid #1e3a5f;border-radius:6px;color:#94a3b8;font-size:12px;outline:none"
      :disabled="sending"/>
    <button @click="wake" :disabled="sending || !mac.trim()"
      style="padding:7px 18px;background:#1d4ed8;color:white;border:none;border-radius:6px;font-size:13px;font-weight:600;cursor:pointer">
      {{ sending ? 'Sending…' : '⚡ Wake' }}
    </button>
  </div>

  <!-- ── Error ──────────────────────────────────────────────────────────── -->
  <div v-if="error" style="padding:8px 14px;background:#450a0a;color:#fca5a5;font-size:12px;border-radius:6px;border:1px solid #7f1d1d;flex-shrink:0">
    ⚠ {{ error }}
  </div>

  <!-- ── Success flash ──────────────────────────────────────────────────── -->
  <div v-if="successMsg" style="padding:8px 14px;background:#052e16;color:#22c55e;font-size:12px;border-radius:6px;border:1px solid #166534;flex-shrink:0;display:flex;align-items:center;gap:8px">
    ✓ {{ successMsg }}
  </div>

  <!-- ── Info box ────────────────────────────────────────────────────────── -->
  <div style="flex-shrink:0;padding:12px 14px;background:#0d1b2a;border-radius:8px;border:1px solid #1e3a5f">
    <div style="font-size:11px;font-weight:700;color:#64748b;text-transform:uppercase;letter-spacing:0.07em;margin-bottom:8px">How it works</div>
    <ul style="margin:0;padding-left:18px;font-size:12px;color:#475569;line-height:2">
      <li>The target device must have Wake-on-LAN enabled in its BIOS/firmware</li>
      <li>Both devices must be on the same local network (or same broadcast domain)</li>
      <li>The magic packet is a UDP broadcast — no response is expected</li>
      <li>MAC formats accepted: <span style="color:#94a3b8;font-family:monospace">AA:BB:CC:DD:EE:FF</span> or <span style="color:#94a3b8;font-family:monospace">AA-BB-CC-DD-EE-FF</span></li>
    </ul>
  </div>

  <!-- ── Saved devices ──────────────────────────────────────────────────── -->
  <div v-if="saved.length" style="flex:1;overflow-y:auto;min-height:0">
    <div style="font-size:11px;font-weight:700;color:#64748b;text-transform:uppercase;letter-spacing:0.07em;margin-bottom:8px;flex-shrink:0">
      Saved Devices
    </div>
    <div style="display:grid;grid-template-columns:repeat(auto-fill,minmax(240px,1fr));gap:8px">
      <div v-for="dev in saved" :key="dev.mac"
        style="padding:12px 14px;border-radius:8px;border:1px solid #1e3a5f;background:#0d1b2a;display:flex;flex-direction:column;gap:6px">
        <div style="display:flex;align-items:center;gap:8px">
          <span style="font-size:18px">💻</span>
          <div style="flex:1">
            <div style="font-size:13px;font-weight:600;color:#e2e8f0">{{ dev.label || 'Device' }}</div>
            <div style="font-size:11px;color:#475569;font-family:monospace">{{ dev.mac }}</div>
          </div>
          <button @click="removeSaved(dev.mac)"
            style="background:none;border:none;color:#334155;cursor:pointer;font-size:14px;padding:2px 4px;line-height:1">✕</button>
        </div>
        <button @click="wakeDevice(dev)"
          style="padding:5px 0;background:#1e3a5f;color:#38bdf8;border:1px solid #1e3a5f;border-radius:6px;font-size:12px;font-weight:600;cursor:pointer">
          ⚡ Wake
        </button>
      </div>
    </div>
  </div>

  <!-- ── Save button (only if mac entered and not saved) ───────────────── -->
  <div v-if="mac.trim() && !isMacSaved" style="flex-shrink:0">
    <button @click="saveCurrent"
      style="padding:6px 14px;background:#1e293b;color:#64748b;border:1px solid #334155;border-radius:6px;font-size:12px;cursor:pointer">
      + Save this device
    </button>
  </div>

</div>
</template>

<script setup>
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const mac        = ref('')
const label      = ref('')
const sending    = ref(false)
const error      = ref(null)
const successMsg = ref(null)

// Saved devices persisted in localStorage
const STORAGE_KEY = 'traceatlas_wol_devices'
const saved = ref(JSON.parse(localStorage.getItem(STORAGE_KEY) ?? '[]'))

const isMacSaved = computed(() =>
  saved.value.some(d => normalise(d.mac) === normalise(mac.value))
)

function normalise(m) { return m.replace(/[^0-9a-fA-F]/g, '').toLowerCase() }

function saveCurrent() {
  const m = mac.value.trim()
  if (!m) return
  saved.value.push({ mac: m, label: label.value.trim() || '' })
  localStorage.setItem(STORAGE_KEY, JSON.stringify(saved.value))
}

function removeSaved(m) {
  saved.value = saved.value.filter(d => d.mac !== m)
  localStorage.setItem(STORAGE_KEY, JSON.stringify(saved.value))
}

async function wakeDevice(dev) {
  mac.value = dev.mac
  await wake()
}

async function wake() {
  const m = mac.value.trim()
  if (!m) return
  sending.value    = true
  error.value      = null
  successMsg.value = null
  try {
    const msg = await invoke('wake_on_lan', { mac: m })
    successMsg.value = msg
    setTimeout(() => { successMsg.value = null }, 4000)
  } catch (e) {
    error.value = String(e)
  } finally {
    sending.value = false
  }
}
</script>
