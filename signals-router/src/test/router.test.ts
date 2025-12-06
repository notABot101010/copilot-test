import { describe, it, expect, beforeEach, vi } from 'vitest';
import { createRouter, NavigationError } from '../router';
import type { RouterOptions, RouteRecord, RouteLocation } from '../types';

// Simple mock for window
const mockPushState = vi.fn();
const mockReplaceState = vi.fn();
const mockBack = vi.fn();
const mockForward = vi.fn();
const mockGo = vi.fn();
const mockAddEventListener = vi.fn();

vi.stubGlobal('window', {
  history: {
    pushState: mockPushState,
    replaceState: mockReplaceState,
    back: mockBack,
    forward: mockForward,
    go: mockGo
  },
  location: {
    pathname: '/',
    search: '',
    hash: '',
    origin: 'http://localhost'
  },
  addEventListener: mockAddEventListener,
  removeEventListener: vi.fn(),
  scrollTo: vi.fn()
});

// ============================================================================
// createRouter Basic Tests
// ============================================================================
describe('createRouter', () => {
  const defaultOptions: RouterOptions = {
    routes: [
      { path: '/', name: 'home' },
      { path: '/users', name: 'users' },
      { path: '/users/:id', name: 'user' },
      { path: '/dashboard', name: 'dashboard', meta: { requiresAuth: true } }
    ]
  };

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should create a router instance', () => {
    const router = createRouter(defaultOptions);
    expect(router).toBeDefined();
    expect(router.currentRoute).toBeDefined();
    expect(router.push).toBeDefined();
    expect(router.replace).toBeDefined();
    expect(router.beforeEach).toBeDefined();
  });

  it('should resolve route by path', () => {
    const router = createRouter(defaultOptions);
    const resolved = router.resolve('/users');
    
    expect(resolved.path).toBe('/users');
    expect(resolved.name).toBe('users');
    expect(resolved.matched.length).toBe(1);
  });

  it('should resolve route by name', () => {
    const router = createRouter(defaultOptions);
    const resolved = router.resolve({ name: 'user', params: { id: '123' } });
    
    expect(resolved.path).toBe('/users/123');
    expect(resolved.params).toEqual({ id: '123' });
  });

  it('should resolve route with query params', () => {
    const router = createRouter(defaultOptions);
    const resolved = router.resolve({ path: '/users', query: { page: '1' } });
    
    expect(resolved.path).toBe('/users');
    expect(resolved.query).toEqual({ page: '1' });
    expect(resolved.fullPath).toBe('/users?page=1');
  });

  it('should extract route params', () => {
    const router = createRouter(defaultOptions);
    const resolved = router.resolve('/users/123');
    
    expect(resolved.params).toEqual({ id: '123' });
    expect(resolved.name).toBe('user');
  });

  it('should merge route meta', () => {
    const router = createRouter(defaultOptions);
    const resolved = router.resolve('/dashboard');
    
    expect(resolved.meta).toEqual({ requiresAuth: true });
  });

  it('should check if route exists', () => {
    const router = createRouter(defaultOptions);
    
    expect(router.hasRoute('home')).toBe(true);
    expect(router.hasRoute('not-found')).toBe(false);
  });

  it('should get all routes', () => {
    const router = createRouter(defaultOptions);
    const routes = router.getRoutes();
    
    expect(routes.length).toBe(4);
  });

  it('should add a new route', () => {
    const router = createRouter(defaultOptions);
    
    router.addRoute(undefined, { path: '/products', name: 'products' });
    
    expect(router.hasRoute('products')).toBe(true);
  });

  it('should remove a route', () => {
    const router = createRouter(defaultOptions);
    
    router.removeRoute('users');
    
    expect(router.hasRoute('users')).toBe(false);
  });

  it('should call history.back()', () => {
    const router = createRouter(defaultOptions);
    router.back();
    expect(mockBack).toHaveBeenCalled();
  });

  it('should call history.forward()', () => {
    const router = createRouter(defaultOptions);
    router.forward();
    expect(mockForward).toHaveBeenCalled();
  });

  it('should call history.go()', () => {
    const router = createRouter(defaultOptions);
    router.go(-2);
    expect(mockGo).toHaveBeenCalledWith(-2);
  });

  it('should register beforeEach guard', () => {
    const router = createRouter(defaultOptions);
    const guard = vi.fn();
    const unregister = router.beforeEach(guard);
    
    expect(typeof unregister).toBe('function');
  });

  it('should register afterEach hook', () => {
    const router = createRouter(defaultOptions);
    const hook = vi.fn();
    const unregister = router.afterEach(hook);
    
    expect(typeof unregister).toBe('function');
  });

  it('should register onError handler', () => {
    const router = createRouter(defaultOptions);
    const handler = vi.fn();
    const unregister = router.onError(handler);
    
    expect(typeof unregister).toBe('function');
  });
});

// ============================================================================
// Router Mode Tests
// ============================================================================
describe('Router modes', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should create router with history mode by default', () => {
    const router = createRouter({
      routes: [{ path: '/', name: 'home' }]
    });
    expect(router.options.mode).toBeUndefined();
  });

  it('should create router with hash mode', () => {
    const router = createRouter({
      routes: [{ path: '/', name: 'home' }],
      mode: 'hash'
    });
    expect(router.options.mode).toBe('hash');
  });

  it('should create router with custom base', () => {
    const router = createRouter({
      routes: [{ path: '/', name: 'home' }],
      base: '/app'
    });
    expect(router.options.base).toBe('/app');
  });
});

// ============================================================================
// Route Resolution Tests
// ============================================================================
describe('Route resolution', () => {
  const routes: RouteRecord[] = [
    { path: '/', name: 'home' },
    { path: '/users', name: 'users' },
    { path: '/users/:id', name: 'user' },
    { path: '/users/:userId/posts/:postId', name: 'user-post' },
    {
      path: '/admin',
      name: 'admin',
      children: [
        { path: '/settings', name: 'admin-settings' },
        { path: '/users/:id', name: 'admin-user' }
      ]
    }
  ];

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should resolve root path', () => {
    const router = createRouter({ routes });
    const resolved = router.resolve('/');
    expect(resolved.path).toBe('/');
    expect(resolved.name).toBe('home');
  });

  it('should resolve path with single param', () => {
    const router = createRouter({ routes });
    const resolved = router.resolve('/users/123');
    expect(resolved.params).toEqual({ id: '123' });
  });

  it('should resolve path with multiple params', () => {
    const router = createRouter({ routes });
    const resolved = router.resolve('/users/123/posts/456');
    expect(resolved.params).toEqual({ userId: '123', postId: '456' });
    expect(resolved.name).toBe('user-post');
  });

  it('should resolve by name with params', () => {
    const router = createRouter({ routes });
    const resolved = router.resolve({ name: 'user', params: { id: '999' } });
    expect(resolved.path).toBe('/users/999');
  });

  it('should resolve by name with query', () => {
    const router = createRouter({ routes });
    const resolved = router.resolve({ name: 'users', query: { page: '2' } });
    expect(resolved.fullPath).toBe('/users?page=2');
  });

  it('should resolve by name with hash', () => {
    const router = createRouter({ routes });
    const resolved = router.resolve({ name: 'home', hash: '#section' });
    expect(resolved.hash).toBe('#section');
    expect(resolved.fullPath).toBe('/#section');
  });

  it('should resolve nested route', () => {
    const router = createRouter({ routes });
    const resolved = router.resolve('/admin/settings');
    expect(resolved.matched.length).toBe(2);
    expect(resolved.matched[0].name).toBe('admin');
    expect(resolved.matched[1].name).toBe('admin-settings');
  });

  it('should resolve nested route with params', () => {
    const router = createRouter({ routes });
    const resolved = router.resolve('/admin/users/42');
    expect(resolved.params).toEqual({ id: '42' });
    expect(resolved.name).toBe('admin-user');
  });

  it('should throw error for non-existent named route', () => {
    const router = createRouter({ routes });
    expect(() => router.resolve({ name: 'non-existent' })).toThrow();
  });

  it('should resolve path with query and hash', () => {
    const router = createRouter({ routes });
    const resolved = router.resolve('/users?search=test#results');
    expect(resolved.path).toBe('/users');
    expect(resolved.query).toEqual({ search: 'test' });
    expect(resolved.hash).toBe('#results');
  });
});

// ============================================================================
// Navigation Tests
// ============================================================================
describe('Navigation', () => {
  const routes: RouteRecord[] = [
    { path: '/', name: 'home' },
    { path: '/about', name: 'about' },
    { path: '/users/:id', name: 'user' }
  ];

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should push to new route', async () => {
    const router = createRouter({ routes });
    router.install();
    
    await router.push('/about');
    
    expect(mockPushState).toHaveBeenCalled();
    expect(router.currentRoute.value.path).toBe('/about');
  });

  it('should replace current route', async () => {
    const router = createRouter({ routes });
    router.install();
    
    await router.replace('/about');
    
    expect(mockReplaceState).toHaveBeenCalled();
    expect(router.currentRoute.value.path).toBe('/about');
  });

  it('should navigate with object syntax', async () => {
    const router = createRouter({ routes });
    router.install();
    
    await router.push({ name: 'user', params: { id: '123' } });
    
    expect(router.currentRoute.value.path).toBe('/users/123');
  });

  it('should handle replace option in object syntax', async () => {
    const router = createRouter({ routes });
    router.install();
    
    await router.push({ path: '/about', replace: true });
    
    expect(mockReplaceState).toHaveBeenCalled();
  });

  it('should not navigate to duplicate route', async () => {
    const router = createRouter({ routes });
    router.install();
    
    await router.push('/about');
    mockPushState.mockClear();
    
    await router.push('/about');
    
    expect(mockPushState).not.toHaveBeenCalled();
  });
});

// ============================================================================
// Navigation Guard Tests
// ============================================================================
describe('Navigation guards', () => {
  const routes: RouteRecord[] = [
    { path: '/', name: 'home' },
    { path: '/login', name: 'login' },
    { path: '/dashboard', name: 'dashboard', meta: { requiresAuth: true } },
    { 
      path: '/protected', 
      name: 'protected',
      beforeEnter: () => false
    }
  ];

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should run beforeEach guard', async () => {
    const router = createRouter({ routes });
    const guard = vi.fn(() => true);
    router.beforeEach(guard);
    router.install();
    
    await router.push('/login');
    
    expect(guard).toHaveBeenCalled();
  });

  it('should block navigation when guard returns false', async () => {
    const router = createRouter({ routes });
    router.beforeEach(() => false);
    router.install();
    
    await expect(router.push('/login')).rejects.toThrow(NavigationError);
  });

  it('should redirect when guard returns route', async () => {
    const router = createRouter({ routes });
    router.beforeEach((to) => {
      if (to.meta?.requiresAuth) {
        return '/login';
      }
    });
    router.install();
    
    await router.push('/dashboard');
    
    expect(router.currentRoute.value.path).toBe('/login');
  });

  it('should unregister beforeEach guard', async () => {
    const router = createRouter({ routes });
    const guard = vi.fn();
    const unregister = router.beforeEach(guard);
    
    unregister();
    router.install();
    await router.push('/login');
    
    expect(guard).not.toHaveBeenCalled();
  });

  it('should run beforeResolve guard', async () => {
    const router = createRouter({ routes });
    const guard = vi.fn();
    router.beforeResolve(guard);
    router.install();
    
    await router.push('/login');
    
    expect(guard).toHaveBeenCalled();
  });

  it('should run afterEach hook', async () => {
    const router = createRouter({ routes });
    const hook = vi.fn();
    router.afterEach(hook);
    router.install();
    
    await router.push('/login');
    
    expect(hook).toHaveBeenCalled();
  });

  it('should pass correct arguments to guard', async () => {
    const router = createRouter({ routes });
    const guard = vi.fn();
    router.beforeEach(guard);
    router.install();
    
    await router.push('/login');
    
    expect(guard).toHaveBeenCalledWith(
      expect.objectContaining({ path: '/login' }),
      expect.objectContaining({ path: '/' })
    );
  });

  it('should handle async guards', async () => {
    const router = createRouter({ routes });
    router.beforeEach(async () => {
      await new Promise(resolve => setTimeout(resolve, 10));
      return true;
    });
    router.install();
    
    await router.push('/login');
    
    expect(router.currentRoute.value.path).toBe('/login');
  });

  it('should run route-specific beforeEnter guard', async () => {
    const router = createRouter({ routes });
    router.install();
    
    await expect(router.push('/protected')).rejects.toThrow(NavigationError);
  });

  it('should handle multiple beforeEach guards', async () => {
    const router = createRouter({ routes });
    const guard1 = vi.fn();
    const guard2 = vi.fn();
    router.beforeEach(guard1);
    router.beforeEach(guard2);
    router.install();
    
    await router.push('/login');
    
    expect(guard1).toHaveBeenCalled();
    expect(guard2).toHaveBeenCalled();
  });
});

// ============================================================================
// Error Handling Tests
// ============================================================================
describe('Error handling', () => {
  const routes: RouteRecord[] = [
    { path: '/', name: 'home' },
    { path: '/error', name: 'error' }
  ];

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should call onError handler on navigation failure', async () => {
    const router = createRouter({ routes });
    const errorHandler = vi.fn();
    router.onError(errorHandler);
    router.beforeEach(() => false);
    router.install();
    
    try {
      await router.push('/error');
    } catch {
      // Expected
    }
    
    expect(errorHandler).toHaveBeenCalled();
  });

  it('should pass error to handler', async () => {
    const router = createRouter({ routes });
    const errorHandler = vi.fn();
    router.onError(errorHandler);
    router.beforeEach(() => false);
    router.install();
    
    try {
      await router.push('/error');
    } catch {
      // Expected
    }
    
    expect(errorHandler).toHaveBeenCalledWith(
      expect.any(NavigationError),
      expect.anything(),
      expect.anything()
    );
  });

  it('should unregister error handler', async () => {
    const router = createRouter({ routes });
    const errorHandler = vi.fn();
    const unregister = router.onError(errorHandler);
    
    unregister();
    router.beforeEach(() => false);
    router.install();
    
    try {
      await router.push('/error');
    } catch {
      // Expected
    }
    
    expect(errorHandler).not.toHaveBeenCalled();
  });
});

// ============================================================================
// NavigationError Tests
// ============================================================================
describe('NavigationError', () => {
  it('should create NavigationError with type aborted', () => {
    const from: RouteLocation = {
      fullPath: '/',
      path: '/',
      params: {},
      query: {},
      hash: '',
      meta: {},
      matched: []
    };
    const to: RouteLocation = {
      fullPath: '/test',
      path: '/test',
      params: {},
      query: {},
      hash: '',
      meta: {},
      matched: []
    };
    
    const error = new NavigationError('Navigation aborted', 'aborted', from, to);
    
    expect(error.message).toBe('Navigation aborted');
    expect(error.type).toBe('aborted');
    expect(error.from).toEqual(from);
    expect(error.to).toEqual(to);
    expect(error.name).toBe('NavigationError');
  });

  it('should create NavigationError with type cancelled', () => {
    const from: RouteLocation = {
      fullPath: '/',
      path: '/',
      params: {},
      query: {},
      hash: '',
      meta: {},
      matched: []
    };
    const to: RouteLocation = {
      fullPath: '/test',
      path: '/test',
      params: {},
      query: {},
      hash: '',
      meta: {},
      matched: []
    };
    
    const error = new NavigationError('Navigation cancelled', 'cancelled', from, to);
    expect(error.type).toBe('cancelled');
  });

  it('should create NavigationError with type redirected', () => {
    const from: RouteLocation = {
      fullPath: '/',
      path: '/',
      params: {},
      query: {},
      hash: '',
      meta: {},
      matched: []
    };
    const to: RouteLocation = {
      fullPath: '/test',
      path: '/test',
      params: {},
      query: {},
      hash: '',
      meta: {},
      matched: []
    };
    
    const error = new NavigationError('Navigation redirected', 'redirected', from, to);
    expect(error.type).toBe('redirected');
  });

  it('should create NavigationError with type duplicated', () => {
    const from: RouteLocation = {
      fullPath: '/',
      path: '/',
      params: {},
      query: {},
      hash: '',
      meta: {},
      matched: []
    };
    const to: RouteLocation = {
      fullPath: '/',
      path: '/',
      params: {},
      query: {},
      hash: '',
      meta: {},
      matched: []
    };
    
    const error = new NavigationError('Navigation duplicated', 'duplicated', from, to);
    expect(error.type).toBe('duplicated');
  });
});

// ============================================================================
// Dynamic Route Management Tests
// ============================================================================
describe('Dynamic route management', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should add route to root', () => {
    const router = createRouter({ routes: [] });
    
    router.addRoute(undefined, { path: '/new', name: 'new' });
    
    expect(router.hasRoute('new')).toBe(true);
    expect(router.getRoutes().length).toBe(1);
  });

  it('should add route as child', () => {
    const router = createRouter({
      routes: [{ path: '/parent', name: 'parent' }]
    });
    
    router.addRoute('parent', { path: '/child', name: 'child' });
    
    expect(router.hasRoute('child')).toBe(true);
    const parentRoute = router.getRoutes()[0];
    expect(parentRoute.children).toBeDefined();
    expect(parentRoute.children!.length).toBe(1);
  });

  it('should return removal function from addRoute', () => {
    const router = createRouter({ routes: [] });
    
    const remove = router.addRoute(undefined, { path: '/temp', name: 'temp' });
    expect(router.hasRoute('temp')).toBe(true);
    
    remove();
    expect(router.hasRoute('temp')).toBe(false);
  });

  it('should remove route by name', () => {
    const router = createRouter({
      routes: [
        { path: '/a', name: 'a' },
        { path: '/b', name: 'b' },
        { path: '/c', name: 'c' }
      ]
    });
    
    router.removeRoute('b');
    
    expect(router.hasRoute('a')).toBe(true);
    expect(router.hasRoute('b')).toBe(false);
    expect(router.hasRoute('c')).toBe(true);
    expect(router.getRoutes().length).toBe(2);
  });

  it('should remove nested route', () => {
    const router = createRouter({
      routes: [
        {
          path: '/parent',
          name: 'parent',
          children: [
            { path: '/child', name: 'child' }
          ]
        }
      ]
    });
    
    router.removeRoute('child');
    
    expect(router.hasRoute('parent')).toBe(true);
    expect(router.hasRoute('child')).toBe(false);
  });

  it('should not fail when removing non-existent route', () => {
    const router = createRouter({ routes: [] });
    
    expect(() => router.removeRoute('non-existent')).not.toThrow();
  });
});

// ============================================================================
// Route Redirect Tests
// ============================================================================
describe('Route redirects', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should handle string redirect', async () => {
    const router = createRouter({
      routes: [
        { path: '/', name: 'home' },
        { path: '/old', redirect: '/new' },
        { path: '/new', name: 'new' }
      ]
    });
    router.install();
    
    await router.push('/old');
    
    expect(router.currentRoute.value.path).toBe('/new');
  });

  it('should handle object redirect', async () => {
    const router = createRouter({
      routes: [
        { path: '/', name: 'home' },
        { path: '/old', redirect: { name: 'new' } },
        { path: '/new', name: 'new' }
      ]
    });
    router.install();
    
    await router.push('/old');
    
    expect(router.currentRoute.value.name).toBe('new');
  });

  it('should handle function redirect', async () => {
    const router = createRouter({
      routes: [
        { path: '/', name: 'home' },
        { 
          path: '/dynamic/:id', 
          redirect: (to) => `/final/${to.params.id}`
        },
        { path: '/final/:id', name: 'final' }
      ]
    });
    router.install();
    
    await router.push('/dynamic/123');
    
    expect(router.currentRoute.value.path).toBe('/final/123');
  });
});

// ============================================================================
// Scroll Behavior Tests
// ============================================================================
describe('Scroll behavior', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should call scrollBehavior on navigation', async () => {
    const scrollBehavior = vi.fn(() => ({ x: 0, y: 0 }));
    const router = createRouter({
      routes: [
        { path: '/', name: 'home' },
        { path: '/page', name: 'page' }
      ],
      scrollBehavior
    });
    router.install();
    
    await router.push('/page');
    
    expect(scrollBehavior).toHaveBeenCalled();
  });

  it('should scroll to position returned by scrollBehavior', async () => {
    const router = createRouter({
      routes: [
        { path: '/', name: 'home' },
        { path: '/page', name: 'page' }
      ],
      scrollBehavior: () => ({ x: 100, y: 200 })
    });
    router.install();
    
    await router.push('/page');
    
    expect(window.scrollTo).toHaveBeenCalledWith(100, 200);
  });

  it('should not scroll if scrollBehavior returns undefined', async () => {
    const router = createRouter({
      routes: [
        { path: '/', name: 'home' },
        { path: '/page', name: 'page' }
      ],
      scrollBehavior: () => undefined
    });
    router.install();
    
    await router.push('/page');
    
    expect(window.scrollTo).not.toHaveBeenCalled();
  });
});

// ============================================================================
// Route Meta Tests
// ============================================================================
describe('Route meta', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should merge meta from parent and child routes', async () => {
    const router = createRouter({
      routes: [
        {
          path: '/admin',
          name: 'admin',
          meta: { requiresAuth: true },
          children: [
            { 
              path: '/settings', 
              name: 'settings',
              meta: { title: 'Settings' }
            }
          ]
        }
      ]
    });
    
    const resolved = router.resolve('/admin/settings');
    
    expect(resolved.meta).toEqual({
      requiresAuth: true,
      title: 'Settings'
    });
  });

  it('should override parent meta with child meta', async () => {
    const router = createRouter({
      routes: [
        {
          path: '/parent',
          name: 'parent',
          meta: { level: 'parent' },
          children: [
            { 
              path: '/child', 
              name: 'child',
              meta: { level: 'child' }
            }
          ]
        }
      ]
    });
    
    const resolved = router.resolve('/parent/child');
    
    expect(resolved.meta.level).toBe('child');
  });

  it('should handle routes without meta', () => {
    const router = createRouter({
      routes: [
        { path: '/no-meta', name: 'no-meta' }
      ]
    });
    
    const resolved = router.resolve('/no-meta');
    
    expect(resolved.meta).toEqual({});
  });
});

// ============================================================================
// Install Tests
// ============================================================================
describe('Router installation', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should initialize current route on install', () => {
    const router = createRouter({
      routes: [{ path: '/', name: 'home' }]
    });
    
    router.install();
    
    expect(router.currentRoute.value.path).toBe('/');
  });

  it('should add popstate listener on install', () => {
    const router = createRouter({
      routes: [{ path: '/', name: 'home' }]
    });
    
    router.install();
    
    expect(mockAddEventListener).toHaveBeenCalledWith('popstate', expect.any(Function));
  });
});
