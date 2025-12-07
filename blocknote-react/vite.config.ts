import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import signalsRouter from '@copilot-test/signals-router/vite'

// https://vite.dev/config/
export default defineConfig({
  plugins: [
    react({
      babel: {
        plugins: [["module:@preact/signals-react-transform"]],
      },
    }),
    signalsRouter(),
  ],
})
