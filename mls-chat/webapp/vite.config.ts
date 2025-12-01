import { defineConfig } from 'vite'
import preact from '@preact/preset-vite'
import tailwindcss from '@tailwindcss/vite'

const API_BASE_URL = process.env.API_BASE_URL || 'http://localhost:3000'

export default defineConfig({
  plugins: [preact(), tailwindcss()],
  server: {
    proxy: {
      '/api': {
        target: API_BASE_URL,
        changeOrigin: true,
      },
    },
  },
  optimizeDeps: {
    exclude: ['openmls-wasm'],
  },
})
