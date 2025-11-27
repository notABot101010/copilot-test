import { createContext } from 'preact';
import { useContext, useEffect, useState } from 'preact/hooks';
import { computed, Signal } from '@preact/signals';
import type { Router, ReactiveRoute, RouteParams, RouteQuery, RouteMeta, RouterContext } from './types';
import { createReactiveRoute } from './router';

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
 */
export function useRoute(): ReactiveRoute {
  const context = useContext(RouterContextProvider);
  if (!context) {
    throw new Error('useRoute must be used within a RouterProvider');
  }
  return context.route;
}

/**
 * Hook to access route params as a signal
 * Usage: const params = useParams(); params.value.website_id
 */
export function useParams(): Signal<RouteParams> {
  const route = useRoute();
  return route.params;
}

/**
 * Hook to access a specific route param
 * Usage: const websiteId = useParam('website_id');
 */
export function useParam(name: string): Signal<string | undefined> {
  const params = useParams();
  return computed(() => params.value[name]) as Signal<string | undefined>;
}

/**
 * Hook to access query params as a signal
 * Usage: const query = useQuery(); query.value.org_id
 */
export function useQuery(): Signal<RouteQuery> {
  const route = useRoute();
  return route.query;
}

/**
 * Hook to access a specific query param
 * Usage: const orgId = useQueryParam('org_id');
 */
export function useQueryParam(name: string): Signal<string | string[] | undefined> {
  const query = useQuery();
  return computed(() => query.value[name]) as Signal<string | string[] | undefined>;
}

/**
 * Hook to access route meta
 */
export function useMeta(): Signal<RouteMeta> {
  const route = useRoute();
  return route.meta;
}

/**
 * Hook to access route path
 */
export function usePath(): Signal<string> {
  const route = useRoute();
  return route.path;
}

/**
 * Hook to access route hash
 */
export function useHash(): Signal<string> {
  const route = useRoute();
  return route.hash;
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
