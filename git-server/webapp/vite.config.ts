import { defineConfig } from 'vite'
import preact from '@preact/preset-vite'
import path from 'path'

// https://vite.dev/config/
export default defineConfig({
  plugins: [preact()],
  build: {
    outDir: path.resolve(__dirname, '../static/dist'),
    emptyOutDir: true,
  },
  server: {
    proxy: {
      '/api': {
        target: process.env.VITE_API_BASE_URL || 'http://localhost:8080',
        changeOrigin: true,
      },
    },
  },
})
