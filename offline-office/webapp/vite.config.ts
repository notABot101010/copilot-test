import { defineConfig } from 'vite'
import preact from '@preact/preset-vite'
import wasm from 'vite-plugin-wasm'
import topLevelAwait from 'vite-plugin-top-level-await'

const API_BASE_URL = process.env.API_BASE_URL || 'http://localhost:8080'

// https://vite.dev/config/
export default defineConfig({
  plugins: [
    wasm(),
    topLevelAwait(),
    preact(),
  ],
  server: {
    proxy: {
      '/api': {
        target: API_BASE_URL,
        changeOrigin: true,
      },
    },
  },
})
