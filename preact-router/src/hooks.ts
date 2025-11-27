import { createContext } from 'preact';
import { useContext } from 'preact/hooks';
import type { Router, ReactiveRoute, RouterContext } from './types';

/**
 * Router context
 */
export const RouterContextProvider = createContext<RouterContext | null>(null);

/**
 * Hook to access the router instance
 */
export function useRouter(): Router {
  const context = useContext(RouterContextProvider);
  if (!context) {
    throw new Error('useRouter must be used within a RouterProvider');
  }
  return context.router;
}

/**
 * Hook to access the reactive route object
 * Returns signals that can be used to subscribe to route changes
 * 
 * Usage:
 *   const route = useRoute();
 *   route.params.value.website_id  // Access route params
 *   route.query.value.org_id       // Access query params
 *   route.path.value               // Access current path
 *   route.hash.value               // Access hash
 *   route.meta.value               // Access route meta
 */
export function useRoute(): ReactiveRoute {
  const context = useContext(RouterContextProvider);
  if (!context) {
    throw new Error('useRoute must be used within a RouterProvider');
  }
  return context.route;
}

/**
 * Hook for navigation
 * Returns push, replace, back, forward, go functions
 */
export function useNavigation() {
  const router = useRouter();
  
  return {
    push: router.push.bind(router),
    replace: router.replace.bind(router),
    back: router.back.bind(router),
    forward: router.forward.bind(router),
    go: router.go.bind(router)
  };
}
