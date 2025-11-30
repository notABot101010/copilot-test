import { defineConfig } from 'vite'

const API_BASE_URL = process.env.API_BASE_URL || 'http://localhost:4001'

export default defineConfig({
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
