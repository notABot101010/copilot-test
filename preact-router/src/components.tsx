import { h, Fragment, ComponentType, VNode, ComponentChildren } from 'preact';
import { useEffect, useState, useMemo, useCallback } from 'preact/hooks';
import { useSignalEffect } from '@preact/signals';
import type { Router, RouterViewProps, RouterLinkProps, RouteComponentProps, RouteRecord, RouterContext } from './types';
import { RouterContextProvider, useRouter, useRoute } from './hooks';
import { createReactiveRoute } from './router';

/**
 * Check if a URL is internal (same origin, not external)
 */
/**
 * Check if a URL is internal (same origin, not external)
 */
function isInternalUrl(href: string, base: string): boolean {
  // Skip obviously non-internal URLs and potentially dangerous schemes
  const lowerHref = href.toLowerCase();
  if (
    lowerHref.startsWith('javascript:') || 
    lowerHref.startsWith('mailto:') || 
    lowerHref.startsWith('tel:') || 
    lowerHref.startsWith('data:') ||
    lowerHref.startsWith('vbscript:') ||
    lowerHref.startsWith('file:')
  ) {
    return false;
  }
  
  try {
    const url = new URL(href, window.location.origin);
    // Check if same origin
    if (url.origin !== window.location.origin) {
      return false;
    }
    // Check if starts with base
    if (base && !url.pathname.startsWith(base)) {
      return false;
    }
    return true;
  } catch {
    // Invalid URLs should not be handled by the router
    return false;
  }
}

/**
 * Router Provider component
 * Wraps the application and provides router context
 * Automatically intercepts clicks on <a> tags for SPA navigation
 */
export interface RouterProviderProps {
  router: Router;
  children?: ComponentChildren;
}

export function RouterProvider({ router, children }: RouterProviderProps): VNode<unknown> {
  const base = router.options.base || '';
  
  // Handle clicks on anchor tags for SPA navigation
  const handleClick = useCallback((event: MouseEvent) => {
    // Find the closest anchor tag
    const target = event.target as HTMLElement;
    const anchor = target.closest('a');
    
    if (!anchor) return;
    
    // Get the href
    const href = anchor.getAttribute('href');
    if (!href) return;
    
    // Skip if modifier keys are pressed (allow open in new tab)
    if (event.ctrlKey || event.metaKey || event.shiftKey || event.altKey) {
      return;
    }
    
    // Skip right-click
    if (event.button !== 0) {
      return;
    }
    
    // Skip if target is _blank or external
    const targetAttr = anchor.getAttribute('target');
    if (targetAttr === '_blank' || targetAttr === '_external') {
      return;
    }
    
    // Skip if download attribute is present
    if (anchor.hasAttribute('download')) {
      return;
    }
    
    // Skip if data-native attribute is present (opt-out of SPA navigation)
    if (anchor.hasAttribute('data-native') || anchor.hasAttribute('data-external')) {
      return;
    }
    
    // Skip external URLs
    if (!isInternalUrl(href, base)) {
      return;
    }
    
    // Skip pure hash-only links that don't change the path (e.g., "#section")
    // These should use native browser scrolling behavior
    if (href.startsWith('#')) {
      // Pure hash links on same page - let browser handle natively
      return;
    }
    
    // Prevent default and use router navigation
    event.preventDefault();
    
    // Check for data-replace attribute
    const replace = anchor.hasAttribute('data-replace');
    
    if (replace) {
      router.replace(href);
    } else {
      router.push(href);
    }
  }, [router, base]);
  
  // Initialize the router and set up link interception
  useEffect(() => {
    router.install();
    
    // Add click listener to document for link interception
    document.addEventListener('click', handleClick);
    
    return () => {
      document.removeEventListener('click', handleClick);
    };
  }, [router, handleClick]);
  
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
    params: route.value.params,
    query: route.value.query
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
    
    const currentPath = route.value.path;
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
