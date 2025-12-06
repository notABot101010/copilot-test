// Types
export type {
  RouteParams,
  RouteQuery,
  RouteLocation,
  RouteMeta,
  RouteRecord,
  RouteComponentProps,
  RouteLocationRaw,
  NavigationGuard,
  NavigationGuardReturn,
  NavigationHookAfter,
  NavigationErrorHandler,
  RouterOptions,
  Router,
  ReactiveRoute,
  RouterContext,
  RouterViewProps,
  RouterLinkProps
} from './types';

// Router
export { createRouter, createReactiveRoute, NavigationError } from './router';

// Hooks
export {
  useRouter,
  useRoute,
  useNavigation
} from './hooks';

// Components
export { RouterProvider, RouterView, RouterLink } from './components';

// Re-export context provider from hooks
export { RouterContextProvider } from './hooks';

// Utilities (for advanced usage)
export {
  parseQuery,
  stringifyQuery,
  normalizePath,
  joinPaths,
  buildPath,
  extractParams,
  matchRoutes
} from './utils';

// Vite plugin
export { signalsRouterPlugin } from './vite-plugin';
