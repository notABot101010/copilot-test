import { describe, it, expect, beforeEach, vi } from 'vitest';
import { createRouter, NavigationError } from '../router';
import type { RouterOptions } from '../types';

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
    hash: ''
  },
  addEventListener: mockAddEventListener,
  removeEventListener: vi.fn(),
  scrollTo: vi.fn()
});

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
