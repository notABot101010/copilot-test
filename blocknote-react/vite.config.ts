import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import { signalsRouterPlugin } from '@copilot-test/signals-router'

// https://vite.dev/config/
export default defineConfig({
  plugins: [
    react({
      babel: {
        plugins: [["module:@preact/signals-react-transform"]],
      },
    }),
    signalsRouterPlugin(),
  ],
})
