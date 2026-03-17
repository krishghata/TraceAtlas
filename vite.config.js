import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'

export default defineConfig({
  plugins: [vue()],

  // Tauri requires a fixed port
  server: {
    port: 5173,
    strictPort: true
  },

  // Don't obscure Rust errors in terminal
  clearScreen: false,

  // Expose Tauri env vars to frontend
  envPrefix: ['VITE_', 'TAURI_'],

  build: {
    target: 'chrome105',
    minify: !process.env.TAURI_DEBUG ? 'esbuild' : false,
    sourcemap: !!process.env.TAURI_DEBUG
  }
})
