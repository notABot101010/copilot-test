/**
 * Vite plugin that automatically configures dedupe for React and signals-react.
 * This prevents multiple instances of React and @preact/signals-react from being bundled,
 * which can cause issues with React hooks and signal subscriptions.
 */
export function signalsRouterPlugin() {
  return {
    name: 'signals-router-dedupe',
    config(config: any) {
      const existingDedupe = config?.resolve?.dedupe || [];
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
