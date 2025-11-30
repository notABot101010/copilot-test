import { defineConfig } from 'vite'
import preact from '@preact/preset-vite'

const API_BASE_URL = process.env.API_BASE_URL || 'http://localhost:3000'

// https://vite.dev/config/
export default defineConfig({
  plugins: [preact()],
  server: {
    proxy: {
      '/api': {
        target: API_BASE_URL,
        changeOrigin: true,
      },
    },
  },
})
