import { defineConfig } from 'vite'
import preact from '@preact/preset-vite'
import tailwindcss from '@tailwindcss/vite';

// https://vite.dev/config/
export default defineConfig({
  plugins: [preact(), tailwindcss()],
  server: {
    proxy: {
      '/api': {
        target: (process.env.VITE_API_BASE_URL || 'http://localhost:8080').replace('http', 'ws'),
        changeOrigin: true,
        ws: true,
      },
    },
  },
})
