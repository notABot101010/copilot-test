import type { Signal } from '@preact/signals-react';
import type { ComponentType, ReactNode } from 'react';

/**
 * Route parameters extracted from the URL path
 */
export type RouteParams = Record<string, string>;

/**
 * Query parameters from the URL
 */
export type RouteQuery = Record<string, string | string[] | undefined>;

/**
 * Route location object representing the current route state
 */
export interface RouteLocation {
  /** The full path including query string */
  fullPath: string;
  /** The path without query string */
  path: string;
  /** Route parameters extracted from the path */
  params: RouteParams;
  /** Query parameters */
  query: RouteQuery;
  /** The hash portion of the URL */
  hash: string;
  /** The name of the matched route */
  name?: string;
  /** Route meta data */
  meta: RouteMeta;
  /** The matched route record */
  matched: RouteRecord[];
}

/**
 * Route meta data that can be attached to routes
 */
export type RouteMeta = Record<string, unknown>;

/**
 * Route record configuration
 */
export interface RouteRecord {
  /** Unique name for the route */
  name?: string;
  /** Path pattern (e.g., '/users/:id') */
  path: string;
  /** Component to render */
  component?: ComponentType<RouteComponentProps>;
  /** Lazy-loaded component */
  lazyComponent?: () => Promise<{ default: ComponentType<RouteComponentProps> }>;
  /** Nested child routes */
  children?: RouteRecord[];
  /** Route meta data */
  meta?: RouteMeta;
  /** Redirect to another route */
  redirect?: string | RouteLocationRaw | ((to: RouteLocation) => string | RouteLocationRaw);
  /** Route-specific guard */
  beforeEnter?: NavigationGuard | NavigationGuard[];
  /** Props to pass to the component */
  props?: boolean | Record<string, unknown> | ((route: RouteLocation) => Record<string, unknown>);
}

/**
 * Props passed to route components
 */
export interface RouteComponentProps {
  params: RouteParams;
  query: RouteQuery;
}

/**
 * Raw location for navigation (string path or location object)
 */
export type RouteLocationRaw = string | {
  name?: string;
  path?: string;
  params?: RouteParams;
  query?: RouteQuery;
  hash?: string;
  replace?: boolean;
};

/**
 * Navigation guard result type
 */
export type NavigationGuardReturn = 
  | void 
  | boolean 
  | string 
  | RouteLocationRaw 
  | Error 
  | Promise<void | boolean | string | RouteLocationRaw | Error>;

/**
 * Navigation guard function signature (similar to Vue Router's beforeEach)
 */
export type NavigationGuard = (
  to: RouteLocation,
  from: RouteLocation
) => NavigationGuardReturn;

/**
 * After navigation hook
 */
export type NavigationHookAfter = (
  to: RouteLocation,
  from: RouteLocation
) => void;

/**
 * Error handler for navigation failures
 */
export type NavigationErrorHandler = (
  error: Error,
  to: RouteLocation,
  from: RouteLocation
) => void;

/**
 * Router options for initialization
 */
export interface RouterOptions {
  /** Route definitions */
  routes: RouteRecord[];
  /** History mode: 'hash' or 'history' (browser) */
  mode?: 'hash' | 'history';
  /** Base path for all routes */
  base?: string;
  /** Scroll behavior on navigation */
  scrollBehavior?: (to: RouteLocation, from: RouteLocation, savedPosition: { x: number; y: number } | null) => { x: number; y: number } | void;
}

/**
 * Router instance interface
 */
export interface Router {
  /** Current route as a signal */
  currentRoute: Signal<RouteLocation>;
  /** Route options */
  options: RouterOptions;
  
  /** Navigate to a new route */
  push(to: RouteLocationRaw): Promise<void>;
  /** Replace current route without adding to history */
  replace(to: RouteLocationRaw): Promise<void>;
  /** Go back in history */
  back(): void;
  /** Go forward in history */
  forward(): void;
  /** Go to a specific point in history */
  go(delta: number): void;
  
  /** Register a global before guard */
  beforeEach(guard: NavigationGuard): () => void;
  /** Register a before resolve guard */
  beforeResolve(guard: NavigationGuard): () => void;
  /** Register an after navigation hook */
  afterEach(hook: NavigationHookAfter): () => void;
  /** Register an error handler */
  onError(handler: NavigationErrorHandler): () => void;
  
  /** Check if a route exists */
  hasRoute(name: string): boolean;
  /** Get all route records */
  getRoutes(): RouteRecord[];
  /** Add a new route */
  addRoute(parentName: string | undefined, route: RouteRecord): () => void;
  /** Remove a route */
  removeRoute(name: string): void;
  /** Resolve a route location */
  resolve(to: RouteLocationRaw): RouteLocation;
  
  /** Install the router (for initialization) */
  install(): void;
}

/**
 * Reactive route object with signal properties (similar to Vue Router's useRoute)
 * The entire route is a single signal, so you access properties via route.value.params, route.value.query, etc.
 */
export type ReactiveRoute = Signal<RouteLocation>;

/**
 * Context value for the router provider
 */
export interface RouterContext {
  router: Router;
  route: ReactiveRoute;
}

/**
 * Router view props
 */
export interface RouterViewProps {
  /** Named view */
  name?: string;
  /** Additional props to pass to the route component */
  props?: Record<string, unknown>;
  /** Component to render when no route matches (NotFound page) */
  notFound?: ComponentType<RouteComponentProps>;
}

/**
 * Router link props
 */
export interface RouterLinkProps {
  /** Target location */
  to: RouteLocationRaw;
  /** Whether to replace instead of push */
  replace?: boolean;
  /** Custom active class */
  activeClass?: string;
  /** Custom exact active class */
  exactActiveClass?: string;
  /** Additional class names */
  class?: string;
  /** Children to render */
  children?: ReactNode;
}
