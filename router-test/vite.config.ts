import { defineConfig } from 'vite'
import preact from '@preact/preset-vite'

// https://vite.dev/config/
export default defineConfig({
  plugins: [preact()],
  resolve: {
    // Ensure a single copy of preact is used across all dependencies
    dedupe: ['preact', '@preact/signals', '@preact/signals-core'],
  },
  optimizeDeps: {
    // Force these packages to be pre-bundled from the app's node_modules
    include: ['preact', 'preact/hooks', '@preact/signals', '@preact/signals-core'],
  }
})
