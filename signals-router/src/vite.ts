/**
 * Vite preset for @copilot-test/signals-router
 * Similar to @preact/preset-vite, this plugin automatically configures
 * everything needed for signals-router to work correctly.
 * 
 * This includes:
 * - Deduplication of react, react-dom, and @preact/signals-react
 * 
 * Usage:
 * ```typescript
 * import { defineConfig } from 'vite'
 * import react from '@vitejs/plugin-react'
 * import signalsRouter from '@copilot-test/signals-router/vite'
 * 
 * export default defineConfig({
 *   plugins: [react(), signalsRouter()],
 * })
 * ```
 */
export default function signalsRouter() {
  return {
    name: 'signals-router-preset',
    // Note: Using 'any' for config parameter to avoid Vite version conflicts
    // between library and consumer project (similar to @preact/preset-vite approach)
    config(config: any) {
      // Safely get existing dedupe array, defaulting to empty array
      const existingDedupe = Array.isArray(config?.resolve?.dedupe) 
        ? config.resolve.dedupe 
        : [];
      const requiredDedupe = ['react', 'react-dom', '@preact/signals-react'];
      
      // Merge existing dedupe with required dedupe, avoiding duplicates
      const mergedDedupe = [...new Set([...existingDedupe, ...requiredDedupe])];
      
      return {
        resolve: {
          dedupe: mergedDedupe
        }
      };
    }
  };
}
