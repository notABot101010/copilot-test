import { defineConfig } from 'vite'
import preact from '@preact/preset-vite'
import tailwindcss from '@tailwindcss/vite'

const WS_SERVER_URL = process.env.WS_SERVER_URL || 'http://localhost:3001'

// https://vite.dev/config/
export default defineConfig({
  plugins: [preact(), tailwindcss()],
  server: {
    proxy: {
      '/ws': {
        target: WS_SERVER_URL,
        changeOrigin: true,
        ws: true,
      },
    },
  },
})
