import { defineConfig } from 'vite'
import preact from '@preact/preset-vite'
import tailwindcss from '@tailwindcss/vite'

const API_BASE_URL = process.env.API_BASE_URL || 'http://localhost:4001'

// https://vite.dev/config/
export default defineConfig({
  plugins: [preact(), tailwindcss()],
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
