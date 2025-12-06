import { signal, Signal } from '@preact/signals';
import type {
  Router,
  RouterOptions,
  RouteLocation,
  RouteLocationRaw,
  RouteRecord,
  NavigationGuard,
  NavigationHookAfter,
  NavigationErrorHandler,
  ReactiveRoute,
  RouteMeta
} from './types';
import {
  parseUrl,
  createFullPath,
  matchRoutes,
  findRouteByName,
  getRoutePathByName,
  buildPath,
  normalizePath,
  joinPaths,
  createEmptyRouteLocation
} from './utils';

/**
 * Navigation error class
 */
export class NavigationError extends Error {
  constructor(
    message: string,
    public readonly type: 'cancelled' | 'aborted' | 'redirected' | 'duplicated',
    public readonly from: RouteLocation,
    public readonly to: RouteLocation
  ) {
    super(message);
    this.name = 'NavigationError';
  }
}

/**
 * Create a router instance (similar to Vue Router's createRouter)
 */
export function createRouter(options: RouterOptions): Router {
  const { routes, mode = 'history', base = '' } = options;

  // Internal state
  const beforeGuards: NavigationGuard[] = [];
  const beforeResolveGuards: NavigationGuard[] = [];
  const afterHooks: NavigationHookAfter[] = [];
  const errorHandlers: NavigationErrorHandler[] = [];

  // Mutable routes list
  let routeRecords: RouteRecord[] = [...routes];

  // Current route signal
  const currentRoute = signal<RouteLocation>(createEmptyRouteLocation());

  // Navigation state
  let isNavigating = false;
  let pendingNavigation: Promise<void> | null = null;

  /**
   * Get the current location from the browser
   */
  function getCurrentLocation(): string {
    if (mode === 'hash') {
      return window.location.hash.slice(1) || '/';
    }
    return window.location.pathname + window.location.search + window.location.hash;
  }

  /**
   * Resolve a route location from a raw location
   */
  function resolve(to: RouteLocationRaw): RouteLocation {
    let path: string;
    let query = {};
    let hash = '';
    let params = {};

    if (typeof to === 'string') {
      const parsed = parseUrl(to, base);
      path = parsed.path;
      query = parsed.query;
      hash = parsed.hash;
    } else {
      if (to.name) {
        // Resolve by name
        const routePath = getRoutePathByName(to.name, routeRecords);
        if (!routePath) {
          throw new Error(`Route with name "${to.name}" not found`);
        }
        path = buildPath(routePath, to.params || {});
        params = to.params || {};
      } else if (to.path) {
        path = normalizePath(to.path);
      } else {
        path = currentRoute.value.path;
      }

      query = to.query || {};
      hash = to.hash || '';
      params = to.params || {};
    }

    // Match the route
    const match = matchRoutes(path, routeRecords);

    if (!match) {
      // No matching route found, return basic location
      return {
        fullPath: createFullPath(path, query, hash),
        path,
        params,
        query,
        hash,
        name: undefined,
        meta: {},
        matched: []
      };
    }

    // Merge params from match
    const mergedParams = { ...match.params, ...params };

    // Get the last matched route's name
    const lastRoute = match.matched[match.matched.length - 1];

    // Merge meta from all matched routes
    const meta: RouteMeta = {};
    for (const route of match.matched) {
      if (route.meta) {
        Object.assign(meta, route.meta);
      }
    }

    return {
      fullPath: createFullPath(path, query, hash),
      path,
      params: mergedParams,
      query,
      hash,
      name: lastRoute?.name,
      meta,
      matched: match.matched
    };
  }

  /**
   * Run navigation guards
   */
  async function runGuards(
    guards: NavigationGuard[],
    to: RouteLocation,
    from: RouteLocation
  ): Promise<RouteLocation | boolean | void> {
    for (const guard of guards) {
      try {
        const result = await guard(to, from);

        if (result === false) {
          return false;
        }

        if (result instanceof Error) {
          throw result;
        }

        if (typeof result === 'string' || (result && typeof result === 'object')) {
          // Redirect
          return resolve(result as RouteLocationRaw);
        }
      } catch (error) {
        throw error;
      }
    }

    return true;
  }

  /**
   * Navigate to a new location
   */
  async function navigate(to: RouteLocationRaw, replace: boolean = false): Promise<void> {
    // Resolve the target location
    const toLocation = resolve(to);
    const fromLocation = currentRoute.value;

    // Check for duplicate navigation
    if (
      toLocation.fullPath === fromLocation.fullPath &&
      JSON.stringify(toLocation.params) === JSON.stringify(fromLocation.params)
    ) {
      return;
    }

    // Prevent concurrent navigations
    if (isNavigating) {
      // Cancel the previous navigation
      isNavigating = false;
    }

    isNavigating = true;

    try {
      // Run before guards
      const beforeResult = await runGuards(beforeGuards, toLocation, fromLocation);

      if (beforeResult === false) {
        throw new NavigationError(
          'Navigation aborted',
          'aborted',
          fromLocation,
          toLocation
        );
      }

      if (beforeResult && typeof beforeResult === 'object' && 'path' in beforeResult) {
        // Redirect
        isNavigating = false;
        await navigate(beforeResult as RouteLocation, replace);
        return;
      }

      // Run route-specific guards
      for (const route of toLocation.matched) {
        if (route.beforeEnter) {
          const guards = Array.isArray(route.beforeEnter)
            ? route.beforeEnter
            : [route.beforeEnter];

          const result = await runGuards(guards, toLocation, fromLocation);

          if (result === false) {
            throw new NavigationError(
              'Navigation aborted by route guard',
              'aborted',
              fromLocation,
              toLocation
            );
          }

          if (result && typeof result === 'object' && 'path' in result) {
            isNavigating = false;
            await navigate(result as RouteLocation, replace);
            return;
          }
        }
      }

      // Run beforeResolve guards
      const resolveResult = await runGuards(beforeResolveGuards, toLocation, fromLocation);

      if (resolveResult === false) {
        throw new NavigationError(
          'Navigation aborted by resolve guard',
          'aborted',
          fromLocation,
          toLocation
        );
      }

      if (resolveResult && typeof resolveResult === 'object' && 'path' in resolveResult) {
        isNavigating = false;
        await navigate(resolveResult as RouteLocation, replace);
        return;
      }

      // Handle redirects
      const lastMatch = toLocation.matched[toLocation.matched.length - 1];
      if (lastMatch?.redirect) {
        let redirectTo: RouteLocationRaw;

        if (typeof lastMatch.redirect === 'function') {
          redirectTo = lastMatch.redirect(toLocation);
        } else {
          redirectTo = lastMatch.redirect;
        }

        isNavigating = false;
        await navigate(redirectTo, replace);
        return;
      }

      // Update the browser history
      const url = mode === 'hash'
        ? `#${toLocation.fullPath}`
        : `${base}${toLocation.fullPath}`;

      // need to copy toLocation without matched because matched can't be copied
      const { matched, ...toLocationCopy } = toLocation;
      if (replace) {
        window.history.replaceState(toLocationCopy, '', url);
      } else {
        window.history.pushState(toLocationCopy, '', url);
      }

      // Update current route
      currentRoute.value = toLocation;

      // Run after hooks
      for (const hook of afterHooks) {
        try {
          hook(toLocation, fromLocation);
        } catch (error) {
          console.error('Error in afterEach hook:', error);
        }
      }

      // Handle scroll behavior
      if (options.scrollBehavior) {
        const savedPosition = null; // Could be implemented with history state
        const scrollPosition = options.scrollBehavior(toLocation, fromLocation, savedPosition);

        if (scrollPosition) {
          window.scrollTo(scrollPosition.x, scrollPosition.y);
        }
      }
    } catch (error) {
      // Handle navigation errors
      for (const handler of errorHandlers) {
        handler(error as Error, toLocation, fromLocation);
      }
      throw error;
    } finally {
      isNavigating = false;
    }
  }

  /**
   * Handle popstate events (browser back/forward)
   */
  function handlePopState(event: PopStateEvent): void {
    const location = getCurrentLocation();
    const toLocation = resolve(location);
    const fromLocation = currentRoute.value;

    // Run guards asynchronously
    (async () => {
      try {
        const beforeResult = await runGuards(beforeGuards, toLocation, fromLocation);

        if (beforeResult === false) {
          // Restore the previous URL
          const url = mode === 'hash'
            ? `#${fromLocation.fullPath}`
            : `${base}${fromLocation.fullPath}`;
          window.history.pushState({ ...fromLocation }, '', url);
          return;
        }

        if (beforeResult && typeof beforeResult === 'object' && 'path' in beforeResult) {
          await navigate(beforeResult as RouteLocation, true);
          return;
        }

        currentRoute.value = toLocation;

        for (const hook of afterHooks) {
          try {
            hook(toLocation, fromLocation);
          } catch (error) {
          }
        }
      } catch (error) {
        for (const handler of errorHandlers) {
          handler(error as Error, toLocation, fromLocation);
        }
      }
    })();
  }

  // Router instance
  const router: Router = {
    currentRoute,
    options,

    async push(to: RouteLocationRaw): Promise<void> {
      const replace = typeof to === 'object' && to.replace === true;
      await navigate(to, replace);
    },

    async replace(to: RouteLocationRaw): Promise<void> {
      await navigate(to, true);
    },

    back(): void {
      window.history.back();
    },

    forward(): void {
      window.history.forward();
    },

    go(delta: number): void {
      window.history.go(delta);
    },

    beforeEach(guard: NavigationGuard): () => void {
      beforeGuards.push(guard);
      return () => {
        const index = beforeGuards.indexOf(guard);
        if (index > -1) {
          beforeGuards.splice(index, 1);
        }
      };
    },

    beforeResolve(guard: NavigationGuard): () => void {
      beforeResolveGuards.push(guard);
      return () => {
        const index = beforeResolveGuards.indexOf(guard);
        if (index > -1) {
          beforeResolveGuards.splice(index, 1);
        }
      };
    },

    afterEach(hook: NavigationHookAfter): () => void {
      afterHooks.push(hook);
      return () => {
        const index = afterHooks.indexOf(hook);
        if (index > -1) {
          afterHooks.splice(index, 1);
        }
      };
    },

    onError(handler: NavigationErrorHandler): () => void {
      errorHandlers.push(handler);
      return () => {
        const index = errorHandlers.indexOf(handler);
        if (index > -1) {
          errorHandlers.splice(index, 1);
        }
      };
    },

    hasRoute(name: string): boolean {
      return findRouteByName(name, routeRecords) !== null;
    },

    getRoutes(): RouteRecord[] {
      return routeRecords;
    },

    addRoute(parentName: string | undefined, route: RouteRecord): () => void {
      if (parentName) {
        const parent = findRouteByName(parentName, routeRecords);
        if (parent) {
          if (!parent.children) {
            parent.children = [];
          }
          parent.children.push(route);
        }
      } else {
        routeRecords.push(route);
      }

      return () => {
        if (route.name) {
          router.removeRoute(route.name);
        }
      };
    },

    removeRoute(name: string): void {
      const removeFromArray = (routes: RouteRecord[]): boolean => {
        const index = routes.findIndex(r => r.name === name);
        if (index > -1) {
          routes.splice(index, 1);
          return true;
        }

        for (const route of routes) {
          if (route.children && removeFromArray(route.children)) {
            return true;
          }
        }

        return false;
      };

      removeFromArray(routeRecords);
    },

    resolve,

    install(): void {
      // Initialize current route from browser location
      const initialLocation = getCurrentLocation();
      currentRoute.value = resolve(initialLocation);

      // Listen for popstate events
      window.addEventListener('popstate', handlePopState);
    }
  };

  return router;
}

/**
 * Create reactive route signal (similar to Vue Router's useRoute)
 * Returns the router's currentRoute signal directly so users can access route.value.params, route.value.query, etc.
 */
export function createReactiveRoute(router: Router): ReactiveRoute {
  return router.currentRoute;
}
