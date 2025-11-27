import { h, Fragment, ComponentType, VNode, ComponentChildren } from 'preact';
import { useEffect, useState, useMemo } from 'preact/hooks';
import { useSignalEffect } from '@preact/signals';
import type { Router, RouterViewProps, RouterLinkProps, RouteComponentProps, RouteRecord, RouterContext } from './types';
import { RouterContextProvider, useRouter, useRoute } from './hooks';
import { createReactiveRoute } from './router';

/**
 * Router Provider component
 * Wraps the application and provides router context
 */
export interface RouterProviderProps {
  router: Router;
  children?: ComponentChildren;
}

export function RouterProvider({ router, children }: RouterProviderProps): VNode<unknown> {
  // Initialize the router on mount
  useEffect(() => {
    router.install();
  }, [router]);
  
  // Create reactive route once
  const route = useMemo(() => createReactiveRoute(router), [router]);
  
  const contextValue: RouterContext = useMemo(() => ({
    router,
    route
  }), [router, route]);
  
  return h(RouterContextProvider.Provider, { value: contextValue }, children) as VNode<unknown>;
}

/**
 * RouterView component
 * Renders the component for the current matched route
 */
export function RouterView({ name, props: additionalProps }: RouterViewProps = {}): VNode<unknown> | null {
  const router = useRouter();
  const route = useRoute();
  
  const [Component, setComponent] = useState<ComponentType<RouteComponentProps> | null>(null);
  const [loading, setLoading] = useState(false);
  
  // Subscribe to route changes using signals
  useSignalEffect(() => {
    const currentRoute = router.currentRoute.value;
    const matched = currentRoute.matched;
    
    if (matched.length === 0) {
      setComponent(null);
      return;
    }
    
    // Get the last matched route (or the one matching the name if specified)
    let matchedRoute: RouteRecord | undefined;
    
    if (name) {
      matchedRoute = matched.find(r => r.name === name);
    } else {
      matchedRoute = matched[matched.length - 1];
    }
    
    if (!matchedRoute) {
      setComponent(null);
      return;
    }
    
    // Handle lazy-loaded components
    if (matchedRoute.lazyComponent) {
      setLoading(true);
      matchedRoute.lazyComponent()
        .then(module => {
          setComponent(() => module.default);
          setLoading(false);
        })
        .catch(error => {
          console.error('Failed to load component:', error);
          setLoading(false);
        });
    } else if (matchedRoute.component) {
      setComponent(() => matchedRoute!.component!);
    } else {
      setComponent(null);
    }
  });
  
  if (loading) {
    return h(Fragment, null) as VNode<unknown>;
  }
  
  if (!Component) {
    return null;
  }
  
  // Get props from route config
  const currentRoute = router.currentRoute.value;
  const matchedRoute = currentRoute.matched[currentRoute.matched.length - 1];
  
  let componentProps: RouteComponentProps = {
    params: route.params.value,
    query: route.query.value
  };
  
  if (matchedRoute?.props) {
    if (typeof matchedRoute.props === 'function') {
      componentProps = { ...componentProps, ...matchedRoute.props(currentRoute) };
    } else if (typeof matchedRoute.props === 'object') {
      componentProps = { ...componentProps, ...matchedRoute.props };
    } else if (matchedRoute.props === true) {
      componentProps = { ...componentProps, ...currentRoute.params };
    }
  }
  
  return h(Component, { ...componentProps, ...additionalProps }) as VNode<unknown>;
}

/**
 * RouterLink component
 * Creates a link that navigates without full page reload
 */
export function RouterLink({
  to,
  replace = false,
  activeClass = 'router-link-active',
  exactActiveClass = 'router-link-exact-active',
  class: className,
  children
}: RouterLinkProps): VNode<unknown> {
  const router = useRouter();
  const route = useRoute();
  
  // Resolve the target location
  const resolved = useMemo(() => {
    try {
      return router.resolve(to);
    } catch {
      return null;
    }
  }, [to, router]);
  
  // Compute active state
  const [isActive, setIsActive] = useState(false);
  const [isExactActive, setIsExactActive] = useState(false);
  
  useSignalEffect(() => {
    if (!resolved) {
      setIsActive(false);
      setIsExactActive(false);
      return;
    }
    
    const currentPath = route.path.value;
    const targetPath = resolved.path;
    
    setIsExactActive(currentPath === targetPath);
    setIsActive(currentPath.startsWith(targetPath));
  });
  
  const handleClick = (event: MouseEvent) => {
    // Don't handle if modifier keys are pressed
    if (event.ctrlKey || event.metaKey || event.shiftKey || event.altKey) {
      return;
    }
    
    // Don't handle right-click
    if (event.button !== 0) {
      return;
    }
    
    event.preventDefault();
    
    if (replace) {
      router.replace(to);
    } else {
      router.push(to);
    }
  };
  
  const href = resolved?.fullPath || (typeof to === 'string' ? to : to.path || '/');
  
  const classes = [
    className,
    isActive ? activeClass : '',
    isExactActive ? exactActiveClass : ''
  ].filter(Boolean).join(' ');
  
  return h('a', {
    href,
    onClick: handleClick,
    class: classes || undefined
  }, children) as VNode<unknown>;
}
