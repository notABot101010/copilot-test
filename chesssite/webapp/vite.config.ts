import { defineConfig } from 'vite'
import preact from '@preact/preset-vite'
import tailwindcss from '@tailwindcss/vite'
import wasm from 'vite-plugin-wasm'

export default defineConfig({
  plugins: [wasm(), preact(), tailwindcss()],
  server: {
    proxy: {
      '/api': {
        target: process.env.API_BASE_URL || 'http://localhost:4001',
        changeOrigin: true,
      },
      '/ws': {
        target: (process.env.API_BASE_URL || 'http://localhost:4001').replace('http', 'ws'),
        ws: true,
      },
    },
  },
})
