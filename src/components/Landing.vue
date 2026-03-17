
<template>
<div style="display:flex;flex-direction:column;align-items:center;justify-content:center;min-height:100vh;background:#0a0f1e;color:white;font-family:'Segoe UI',sans-serif;text-align:center;padding:40px">

  <img src="/logo.svg" alt="TraceAtlas" style="width:160px;height:160px;margin-bottom:28px;filter:drop-shadow(0 0 24px rgba(56,189,248,0.45))"/>

  <h1 style="font-size:36px;font-weight:700;color:#38bdf8;margin:0 0 10px;letter-spacing:-0.5px">TraceAtlas</h1>
  <p style="font-size:15px;color:#64748b;margin:0 0 36px;max-width:380px;line-height:1.6">
    Visualize how your internet traffic travels — hop by hop, country by country, across the globe.
  </p>

  <button @click="$emit('start')"
    style="padding:12px 36px;background:#1d4ed8;color:white;border:none;border-radius:8px;font-size:15px;font-weight:600;cursor:pointer;letter-spacing:0.3px;transition:background 0.15s">
    Start Tracing
  </button>

  <p style="margin-top:52px;font-size:11px;color:#1e3a5f">Tauri · Vue 3 · Leaflet · ip-api.com</p>
</div>
</template>
