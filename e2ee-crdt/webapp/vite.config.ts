import { defineConfig } from 'vite'
import preact from '@preact/preset-vite'
import tailwindcss from '@tailwindcss/vite'
import wasm from 'vite-plugin-wasm'

const API_BASE_URL = process.env.API_BASE_URL || 'http://localhost:3001'

// https://vite.dev/config/
export default defineConfig({
  plugins: [wasm(), preact(), tailwindcss()],
  server: {
    proxy: {
      '/api': {
        target: API_BASE_URL,
        changeOrigin: true,
      },
      '/ws': {
        target: API_BASE_URL.replace('http', 'ws'),
        ws: true,
      },
    },
  },
})
