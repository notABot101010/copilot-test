import { describe, it, expect, vi, beforeEach } from 'vitest';
import { h } from 'preact';
import { render } from '@testing-library/preact';
import { signal } from '@preact/signals';
import { RouterContextProvider, useRouter, useRoute, useNavigation } from '../hooks';
import type { Router, ReactiveRoute, RouterContext, RouteLocation } from '../types';

// Helper to create a mock router
function createMockRouter(overrides: Partial<Router> = {}): Router {
  const currentRoute = signal<RouteLocation>({
    fullPath: '/',
    path: '/',
    params: {},
    query: {},
    hash: '',
    meta: {},
    matched: []
  });

  return {
    currentRoute,
    options: { routes: [] },
    push: vi.fn(),
    replace: vi.fn(),
    back: vi.fn(),
    forward: vi.fn(),
    go: vi.fn(),
    beforeEach: vi.fn(() => () => {}),
    beforeResolve: vi.fn(() => () => {}),
    afterEach: vi.fn(() => () => {}),
    onError: vi.fn(() => () => {}),
    hasRoute: vi.fn(),
    getRoutes: vi.fn(() => []),
    addRoute: vi.fn(() => () => {}),
    removeRoute: vi.fn(),
    resolve: vi.fn(),
    install: vi.fn(),
    ...overrides
  };
}

// ============================================================================
// useRouter Tests
// ============================================================================
describe('useRouter', () => {
  it('should throw error when used outside RouterProvider', () => {
    const TestComponent = () => {
      try {
        useRouter();
        return h('div', null, 'should not reach');
      } catch (err) {
        return h('div', null, (err as Error).message);
      }
    };

    const { container } = render(h(TestComponent, null));
    expect(container.textContent).toContain('useRouter must be used within a RouterProvider');
  });

  it('should return router instance when inside RouterProvider', () => {
    const mockRouter = createMockRouter();
    const mockRoute = mockRouter.currentRoute;
    let routerInstance: Router | null = null;

    const TestComponent = () => {
      routerInstance = useRouter();
      return h('div', null, 'test');
    };

    const context: RouterContext = {
      router: mockRouter,
      route: mockRoute
    };

    render(
      h(RouterContextProvider.Provider, { value: context },
        h(TestComponent, null)
      )
    );

    expect(routerInstance).toBe(mockRouter);
  });

  it('should provide access to router push method', () => {
    const mockRouter = createMockRouter();
    let pushFn: typeof mockRouter.push | null = null;

    const TestComponent = () => {
      const router = useRouter();
      pushFn = router.push;
      return h('div', null, 'test');
    };

    const context: RouterContext = {
      router: mockRouter,
      route: mockRouter.currentRoute
    };

    render(
      h(RouterContextProvider.Provider, { value: context },
        h(TestComponent, null)
      )
    );

    expect(pushFn).toBe(mockRouter.push);
  });

  it('should provide access to router replace method', () => {
    const mockRouter = createMockRouter();
    let replaceFn: typeof mockRouter.replace | null = null;

    const TestComponent = () => {
      const router = useRouter();
      replaceFn = router.replace;
      return h('div', null, 'test');
    };

    const context: RouterContext = {
      router: mockRouter,
      route: mockRouter.currentRoute
    };

    render(
      h(RouterContextProvider.Provider, { value: context },
        h(TestComponent, null)
      )
    );

    expect(replaceFn).toBe(mockRouter.replace);
  });

  it('should provide access to router options', () => {
    const mockRouter = createMockRouter({
      options: { routes: [], mode: 'hash', base: '/app' }
    });
    let options: typeof mockRouter.options | null = null;

    const TestComponent = () => {
      const router = useRouter();
      options = router.options;
      return h('div', null, 'test');
    };

    const context: RouterContext = {
      router: mockRouter,
      route: mockRouter.currentRoute
    };

    render(
      h(RouterContextProvider.Provider, { value: context },
        h(TestComponent, null)
      )
    );

    expect(options).toEqual({ routes: [], mode: 'hash', base: '/app' });
  });
});

// ============================================================================
// useRoute Tests
// ============================================================================
describe('useRoute', () => {
  it('should throw error when used outside RouterProvider', () => {
    const TestComponent = () => {
      try {
        useRoute();
        return h('div', null, 'should not reach');
      } catch (err) {
        return h('div', null, (err as Error).message);
      }
    };

    const { container } = render(h(TestComponent, null));
    expect(container.textContent).toContain('useRoute must be used within a RouterProvider');
  });

  it('should return route signal when inside RouterProvider', () => {
    const mockRouter = createMockRouter();
    let routeSignal: ReactiveRoute | null = null;

    const TestComponent = () => {
      routeSignal = useRoute();
      return h('div', null, 'test');
    };

    const context: RouterContext = {
      router: mockRouter,
      route: mockRouter.currentRoute
    };

    render(
      h(RouterContextProvider.Provider, { value: context },
        h(TestComponent, null)
      )
    );

    expect(routeSignal).toBe(mockRouter.currentRoute);
  });

  it('should provide access to route path', () => {
    const currentRoute = signal<RouteLocation>({
      fullPath: '/users/123',
      path: '/users/123',
      params: { id: '123' },
      query: {},
      hash: '',
      meta: {},
      matched: []
    });

    const mockRouter = createMockRouter({ currentRoute });
    let path: string | null = null;

    const TestComponent = () => {
      const route = useRoute();
      path = route.value.path;
      return h('div', null, 'test');
    };

    const context: RouterContext = {
      router: mockRouter,
      route: currentRoute
    };

    render(
      h(RouterContextProvider.Provider, { value: context },
        h(TestComponent, null)
      )
    );

    expect(path).toBe('/users/123');
  });

  it('should provide access to route params', () => {
    const currentRoute = signal<RouteLocation>({
      fullPath: '/users/456',
      path: '/users/456',
      params: { id: '456' },
      query: {},
      hash: '',
      meta: {},
      matched: []
    });

    const mockRouter = createMockRouter({ currentRoute });
    let params: Record<string, string> | null = null;

    const TestComponent = () => {
      const route = useRoute();
      params = route.value.params;
      return h('div', null, 'test');
    };

    const context: RouterContext = {
      router: mockRouter,
      route: currentRoute
    };

    render(
      h(RouterContextProvider.Provider, { value: context },
        h(TestComponent, null)
      )
    );

    expect(params).toEqual({ id: '456' });
  });

  it('should provide access to route query', () => {
    const currentRoute = signal<RouteLocation>({
      fullPath: '/search?q=test&page=1',
      path: '/search',
      params: {},
      query: { q: 'test', page: '1' },
      hash: '',
      meta: {},
      matched: []
    });

    const mockRouter = createMockRouter({ currentRoute });
    let query: Record<string, string | string[] | undefined> | null = null;

    const TestComponent = () => {
      const route = useRoute();
      query = route.value.query;
      return h('div', null, 'test');
    };

    const context: RouterContext = {
      router: mockRouter,
      route: currentRoute
    };

    render(
      h(RouterContextProvider.Provider, { value: context },
        h(TestComponent, null)
      )
    );

    expect(query).toEqual({ q: 'test', page: '1' });
  });

  it('should provide access to route hash', () => {
    const currentRoute = signal<RouteLocation>({
      fullPath: '/page#section',
      path: '/page',
      params: {},
      query: {},
      hash: '#section',
      meta: {},
      matched: []
    });

    const mockRouter = createMockRouter({ currentRoute });
    let hash: string | null = null;

    const TestComponent = () => {
      const route = useRoute();
      hash = route.value.hash;
      return h('div', null, 'test');
    };

    const context: RouterContext = {
      router: mockRouter,
      route: currentRoute
    };

    render(
      h(RouterContextProvider.Provider, { value: context },
        h(TestComponent, null)
      )
    );

    expect(hash).toBe('#section');
  });

  it('should provide access to route meta', () => {
    const currentRoute = signal<RouteLocation>({
      fullPath: '/dashboard',
      path: '/dashboard',
      params: {},
      query: {},
      hash: '',
      meta: { requiresAuth: true, title: 'Dashboard' },
      matched: []
    });

    const mockRouter = createMockRouter({ currentRoute });
    let meta: Record<string, unknown> | null = null;

    const TestComponent = () => {
      const route = useRoute();
      meta = route.value.meta;
      return h('div', null, 'test');
    };

    const context: RouterContext = {
      router: mockRouter,
      route: currentRoute
    };

    render(
      h(RouterContextProvider.Provider, { value: context },
        h(TestComponent, null)
      )
    );

    expect(meta).toEqual({ requiresAuth: true, title: 'Dashboard' });
  });

  it('should provide access to route name', () => {
    const currentRoute = signal<RouteLocation>({
      fullPath: '/home',
      path: '/home',
      params: {},
      query: {},
      hash: '',
      name: 'home',
      meta: {},
      matched: []
    });

    const mockRouter = createMockRouter({ currentRoute });
    let name: string | undefined = undefined;

    const TestComponent = () => {
      const route = useRoute();
      name = route.value.name;
      return h('div', null, 'test');
    };

    const context: RouterContext = {
      router: mockRouter,
      route: currentRoute
    };

    render(
      h(RouterContextProvider.Provider, { value: context },
        h(TestComponent, null)
      )
    );

    expect(name).toBe('home');
  });

  it('should provide access to fullPath', () => {
    const currentRoute = signal<RouteLocation>({
      fullPath: '/search?q=hello#results',
      path: '/search',
      params: {},
      query: { q: 'hello' },
      hash: '#results',
      meta: {},
      matched: []
    });

    const mockRouter = createMockRouter({ currentRoute });
    let fullPath: string | null = null;

    const TestComponent = () => {
      const route = useRoute();
      fullPath = route.value.fullPath;
      return h('div', null, 'test');
    };

    const context: RouterContext = {
      router: mockRouter,
      route: currentRoute
    };

    render(
      h(RouterContextProvider.Provider, { value: context },
        h(TestComponent, null)
      )
    );

    expect(fullPath).toBe('/search?q=hello#results');
  });
});

// ============================================================================
// useNavigation Tests
// ============================================================================
describe('useNavigation', () => {
  it('should return navigation methods', () => {
    const mockRouter = createMockRouter();
    let navigation: ReturnType<typeof useNavigation> | null = null;

    const TestComponent = () => {
      navigation = useNavigation();
      return h('div', null, 'test');
    };

    const context: RouterContext = {
      router: mockRouter,
      route: mockRouter.currentRoute
    };

    render(
      h(RouterContextProvider.Provider, { value: context },
        h(TestComponent, null)
      )
    );

    expect(navigation).not.toBeNull();
    expect(typeof navigation!.push).toBe('function');
    expect(typeof navigation!.replace).toBe('function');
    expect(typeof navigation!.back).toBe('function');
    expect(typeof navigation!.forward).toBe('function');
    expect(typeof navigation!.go).toBe('function');
  });

  it('should bind push to router', () => {
    const mockRouter = createMockRouter();
    let navigation: ReturnType<typeof useNavigation> | null = null;

    const TestComponent = () => {
      navigation = useNavigation();
      return h('div', null, 'test');
    };

    const context: RouterContext = {
      router: mockRouter,
      route: mockRouter.currentRoute
    };

    render(
      h(RouterContextProvider.Provider, { value: context },
        h(TestComponent, null)
      )
    );

    navigation!.push('/test');
    expect(mockRouter.push).toHaveBeenCalledWith('/test');
  });

  it('should bind replace to router', () => {
    const mockRouter = createMockRouter();
    let navigation: ReturnType<typeof useNavigation> | null = null;

    const TestComponent = () => {
      navigation = useNavigation();
      return h('div', null, 'test');
    };

    const context: RouterContext = {
      router: mockRouter,
      route: mockRouter.currentRoute
    };

    render(
      h(RouterContextProvider.Provider, { value: context },
        h(TestComponent, null)
      )
    );

    navigation!.replace('/test');
    expect(mockRouter.replace).toHaveBeenCalledWith('/test');
  });

  it('should bind back to router', () => {
    const mockRouter = createMockRouter();
    let navigation: ReturnType<typeof useNavigation> | null = null;

    const TestComponent = () => {
      navigation = useNavigation();
      return h('div', null, 'test');
    };

    const context: RouterContext = {
      router: mockRouter,
      route: mockRouter.currentRoute
    };

    render(
      h(RouterContextProvider.Provider, { value: context },
        h(TestComponent, null)
      )
    );

    navigation!.back();
    expect(mockRouter.back).toHaveBeenCalled();
  });

  it('should bind forward to router', () => {
    const mockRouter = createMockRouter();
    let navigation: ReturnType<typeof useNavigation> | null = null;

    const TestComponent = () => {
      navigation = useNavigation();
      return h('div', null, 'test');
    };

    const context: RouterContext = {
      router: mockRouter,
      route: mockRouter.currentRoute
    };

    render(
      h(RouterContextProvider.Provider, { value: context },
        h(TestComponent, null)
      )
    );

    navigation!.forward();
    expect(mockRouter.forward).toHaveBeenCalled();
  });

  it('should bind go to router', () => {
    const mockRouter = createMockRouter();
    let navigation: ReturnType<typeof useNavigation> | null = null;

    const TestComponent = () => {
      navigation = useNavigation();
      return h('div', null, 'test');
    };

    const context: RouterContext = {
      router: mockRouter,
      route: mockRouter.currentRoute
    };

    render(
      h(RouterContextProvider.Provider, { value: context },
        h(TestComponent, null)
      )
    );

    navigation!.go(-2);
    expect(mockRouter.go).toHaveBeenCalledWith(-2);
  });

  it('should call push with object navigation', () => {
    const mockRouter = createMockRouter();
    let navigation: ReturnType<typeof useNavigation> | null = null;

    const TestComponent = () => {
      navigation = useNavigation();
      return h('div', null, 'test');
    };

    const context: RouterContext = {
      router: mockRouter,
      route: mockRouter.currentRoute
    };

    render(
      h(RouterContextProvider.Provider, { value: context },
        h(TestComponent, null)
      )
    );

    navigation!.push({ name: 'user', params: { id: '123' } });
    expect(mockRouter.push).toHaveBeenCalledWith({ name: 'user', params: { id: '123' } });
  });

  it('should call replace with query params', () => {
    const mockRouter = createMockRouter();
    let navigation: ReturnType<typeof useNavigation> | null = null;

    const TestComponent = () => {
      navigation = useNavigation();
      return h('div', null, 'test');
    };

    const context: RouterContext = {
      router: mockRouter,
      route: mockRouter.currentRoute
    };

    render(
      h(RouterContextProvider.Provider, { value: context },
        h(TestComponent, null)
      )
    );

    navigation!.replace({ path: '/search', query: { q: 'test' } });
    expect(mockRouter.replace).toHaveBeenCalledWith({ path: '/search', query: { q: 'test' } });
  });
});

// ============================================================================
// RouterContextProvider Tests
// ============================================================================
describe('RouterContextProvider', () => {
  it('should be a valid context provider', () => {
    expect(RouterContextProvider).toBeDefined();
    expect(RouterContextProvider.Provider).toBeDefined();
  });

  it('should provide null by default', () => {
    let contextValue: RouterContext | null = 'initial' as unknown as RouterContext | null;

    const TestComponent = () => {
      try {
        useRouter();
      } catch {
        contextValue = null;
      }
      return h('div', null, 'test');
    };

    render(h(TestComponent, null));
    expect(contextValue).toBeNull();
  });

  it('should provide context value to children', () => {
    const mockRouter = createMockRouter();
    let receivedRouter: Router | null = null;

    const TestComponent = () => {
      receivedRouter = useRouter();
      return h('div', null, 'test');
    };

    const context: RouterContext = {
      router: mockRouter,
      route: mockRouter.currentRoute
    };

    render(
      h(RouterContextProvider.Provider, { value: context },
        h(TestComponent, null)
      )
    );

    expect(receivedRouter).toBe(mockRouter);
  });

  it('should provide context to deeply nested children', () => {
    const mockRouter = createMockRouter();
    let receivedRouter: Router | null = null;

    const DeepChild = () => {
      receivedRouter = useRouter();
      return h('span', null, 'deep');
    };

    const MiddleChild = () => {
      return h('div', null, h(DeepChild, null));
    };

    const context: RouterContext = {
      router: mockRouter,
      route: mockRouter.currentRoute
    };

    render(
      h(RouterContextProvider.Provider, { value: context },
        h(MiddleChild, null)
      )
    );

    expect(receivedRouter).toBe(mockRouter);
  });
});
