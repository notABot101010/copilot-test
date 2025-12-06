/**
 * Vite plugin that automatically configures dedupe for React and signals-react.
 * This prevents multiple instances of React and @preact/signals-react from being bundled,
 * which can cause issues with React hooks and signal subscriptions.
 */
export function signalsRouterPlugin() {
  return {
    name: 'signals-router-dedupe',
    config() {
      return {
        resolve: {
          dedupe: ['react', 'react-dom', '@preact/signals-react']
        }
      };
    }
  };
}
