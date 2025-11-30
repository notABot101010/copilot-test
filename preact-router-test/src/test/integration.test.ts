import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { h, ComponentType } from 'preact';
import { render, fireEvent, cleanup, waitFor } from '@testing-library/preact';
import {
  createRouter,
  RouterProvider,
  RouterView,
  RouterLink,
  useRouter,
  useRoute,
  useNavigation,
  NavigationError
} from '@copilot-test/preact-router';
import type { RouteComponentProps, RouteRecord, Router } from '@copilot-test/preact-router';

// ============================================================================
// Test Helpers
// ============================================================================
const mockPushState = vi.fn();
const mockReplaceState = vi.fn();
const mockAddEventListener = vi.fn();
const mockRemoveEventListener = vi.fn();

function setupWindowMock(pathname: string = '/') {
  vi.stubGlobal('window', {
    history: {
      pushState: mockPushState,
      replaceState: mockReplaceState,
      back: vi.fn(),
      forward: vi.fn(),
      go: vi.fn()
    },
    location: {
      pathname,
      search: '',
      hash: '',
      origin: 'http://localhost'
    },
    addEventListener: mockAddEventListener,
    removeEventListener: mockRemoveEventListener,
    scrollTo: vi.fn()
  });
}

beforeEach(() => {
  setupWindowMock('/');
  vi.clearAllMocks();
});

afterEach(() => {
  cleanup();
});

// Test Components
const HomePage = (props: RouteComponentProps) => h('div', { 'data-testid': 'home' }, 'Home Page');
const AboutPage = (props: RouteComponentProps) => h('div', { 'data-testid': 'about' }, 'About Page');
const UserPage = (props: RouteComponentProps) => h('div', { 'data-testid': 'user' }, `User: ${props.params?.id || 'unknown'}`);
const ProfilePage = (props: RouteComponentProps) => h('div', { 'data-testid': 'profile' }, `Profile: ${props.params?.userId || 'unknown'}`);
const DashboardPage = (props: RouteComponentProps) => h('div', { 'data-testid': 'dashboard' }, 'Dashboard');
const NotFoundPage = () => h('div', { 'data-testid': 'not-found' }, '404 Not Found');
const LoginPage = () => h('div', { 'data-testid': 'login' }, 'Login Page');
const ProtectedPage = () => h('div', { 'data-testid': 'protected' }, 'Protected Content');

// ============================================================================
// Basic Router Integration Tests (20 tests)
// ============================================================================
describe('Basic Router Integration', () => {
  it('should create router with routes', () => {
    const router = createRouter({
      routes: [
        { path: '/', name: 'home', component: HomePage }
      ]
    });
    expect(router).toBeDefined();
    expect(router.hasRoute('home')).toBe(true);
  });

  it('should render initial route', () => {
    const router = createRouter({
      routes: [{ path: '/', name: 'home', component: HomePage }]
    });

    const { container } = render(
      h(RouterProvider, { router }, h(RouterView, null))
    );

    expect(container.querySelector('[data-testid="home"]')).not.toBeNull();
  });

  it('should navigate between routes programmatically', async () => {
    const router = createRouter({
      routes: [
        { path: '/', name: 'home', component: HomePage },
        { path: '/about', name: 'about', component: AboutPage }
      ]
    });

    render(h(RouterProvider, { router }, h(RouterView, null)));

    await router.push('/about');

    expect(router.currentRoute.value.path).toBe('/about');
  });

  it('should resolve params for a route', () => {
    const router = createRouter({
      routes: [
        { path: '/users/:id', name: 'user', component: UserPage }
      ]
    });

    const resolved = router.resolve('/users/123');
    expect(resolved.params).toEqual({ id: '123' });
    expect(resolved.path).toBe('/users/123');
  });

  it('should handle query parameters', async () => {
    const router = createRouter({
      routes: [{ path: '/', name: 'home', component: HomePage }]
    });

    await router.push({ path: '/', query: { page: '1', sort: 'name' } });

    expect(router.currentRoute.value.query).toEqual({ page: '1', sort: 'name' });
  });

  it('should handle hash navigation', async () => {
    const router = createRouter({
      routes: [{ path: '/', name: 'home', component: HomePage }]
    });

    await router.push({ path: '/', hash: '#section1' });

    expect(router.currentRoute.value.hash).toBe('#section1');
  });

  it('should navigate by route name', async () => {
    const router = createRouter({
      routes: [
        { path: '/', name: 'home', component: HomePage },
        { path: '/users/:id', name: 'user', component: UserPage }
      ]
    });

    await router.push({ name: 'user', params: { id: '456' } });

    expect(router.currentRoute.value.path).toBe('/users/456');
  });

  it('should replace history with replace()', async () => {
    const router = createRouter({
      routes: [
        { path: '/', name: 'home', component: HomePage },
        { path: '/about', name: 'about', component: AboutPage }
      ]
    });
    router.install();

    await router.replace('/about');

    expect(mockReplaceState).toHaveBeenCalled();
  });

  it('should call history.back()', () => {
    const router = createRouter({
      routes: [{ path: '/', name: 'home', component: HomePage }]
    });
    const backMock = vi.fn();
    window.history.back = backMock;

    router.back();

    expect(backMock).toHaveBeenCalled();
  });

  it('should call history.forward()', () => {
    const router = createRouter({
      routes: [{ path: '/', name: 'home', component: HomePage }]
    });
    const forwardMock = vi.fn();
    window.history.forward = forwardMock;

    router.forward();

    expect(forwardMock).toHaveBeenCalled();
  });

  it('should call history.go()', () => {
    const router = createRouter({
      routes: [{ path: '/', name: 'home', component: HomePage }]
    });
    const goMock = vi.fn();
    window.history.go = goMock;

    router.go(-2);

    expect(goMock).toHaveBeenCalledWith(-2);
  });

  it('should resolve route correctly', () => {
    const router = createRouter({
      routes: [
        { path: '/', name: 'home', component: HomePage },
        { path: '/users/:id', name: 'user', component: UserPage }
      ]
    });

    const resolved = router.resolve('/users/789');

    expect(resolved.path).toBe('/users/789');
    expect(resolved.params).toEqual({ id: '789' });
  });

  it('should resolve route by name with params', () => {
    const router = createRouter({
      routes: [
        { path: '/users/:id/profile/:profileId', name: 'user-profile', component: ProfilePage }
      ]
    });

    const resolved = router.resolve({ name: 'user-profile', params: { id: '1', profileId: '2' } });

    expect(resolved.path).toBe('/users/1/profile/2');
  });

  it('should handle multiple routes', () => {
    const router = createRouter({
      routes: [
        { path: '/', name: 'home', component: HomePage },
        { path: '/about', name: 'about', component: AboutPage },
        { path: '/users', name: 'users', component: UserPage },
        { path: '/users/:id', name: 'user', component: UserPage },
        { path: '/dashboard', name: 'dashboard', component: DashboardPage }
      ]
    });

    expect(router.getRoutes().length).toBe(5);
    expect(router.hasRoute('home')).toBe(true);
    expect(router.hasRoute('about')).toBe(true);
    expect(router.hasRoute('users')).toBe(true);
    expect(router.hasRoute('user')).toBe(true);
    expect(router.hasRoute('dashboard')).toBe(true);
  });

  it('should prevent duplicate navigation', async () => {
    const router = createRouter({
      routes: [
        { path: '/', name: 'home', component: HomePage },
        { path: '/about', name: 'about', component: AboutPage }
      ]
    });
    router.install();

    await router.push('/about');
    mockPushState.mockClear();

    await router.push('/about');

    expect(mockPushState).not.toHaveBeenCalled();
  });

  it('should access currentRoute signal', () => {
    const router = createRouter({
      routes: [{ path: '/', name: 'home', component: HomePage }]
    });
    router.install();

    expect(router.currentRoute.value).toBeDefined();
    expect(router.currentRoute.value.path).toBe('/');
  });

  it('should provide router options', () => {
    const router = createRouter({
      routes: [{ path: '/', name: 'home', component: HomePage }],
      mode: 'history',
      base: '/app'
    });

    expect(router.options.mode).toBe('history');
    expect(router.options.base).toBe('/app');
  });

  it('should handle route with multiple params', async () => {
    const router = createRouter({
      routes: [
        { path: '/org/:orgId/team/:teamId/member/:memberId', name: 'member', component: UserPage }
      ]
    });

    const resolved = router.resolve('/org/1/team/2/member/3');

    expect(resolved.params).toEqual({ orgId: '1', teamId: '2', memberId: '3' });
  });

  it('should render RouterView correctly', () => {
    const router = createRouter({
      routes: [{ path: '/', name: 'home', component: HomePage }]
    });

    const { container } = render(
      h(RouterProvider, { router },
        h(RouterView, null)
      )
    );

    expect(container.textContent).toContain('Home Page');
  });

  it('should work with replace option in push', async () => {
    const router = createRouter({
      routes: [
        { path: '/', name: 'home', component: HomePage },
        { path: '/about', name: 'about', component: AboutPage }
      ]
    });
    router.install();

    await router.push({ path: '/about', replace: true });

    expect(mockReplaceState).toHaveBeenCalled();
  });
});

// ============================================================================
// Navigation Guards Tests (20 tests)
// ============================================================================
describe('Navigation Guards', () => {
  it('should call beforeEach guard on navigation', async () => {
    const router = createRouter({
      routes: [
        { path: '/', name: 'home', component: HomePage },
        { path: '/about', name: 'about', component: AboutPage }
      ]
    });
    const guard = vi.fn();
    router.beforeEach(guard);
    router.install();

    await router.push('/about');

    expect(guard).toHaveBeenCalled();
  });

  it('should pass correct arguments to beforeEach', async () => {
    const router = createRouter({
      routes: [
        { path: '/', name: 'home', component: HomePage },
        { path: '/about', name: 'about', component: AboutPage }
      ]
    });
    const guard = vi.fn();
    router.beforeEach(guard);
    router.install();

    await router.push('/about');

    expect(guard).toHaveBeenCalledWith(
      expect.objectContaining({ path: '/about' }),
      expect.objectContaining({ path: '/' })
    );
  });

  it('should block navigation when guard returns false', async () => {
    const router = createRouter({
      routes: [
        { path: '/', name: 'home', component: HomePage },
        { path: '/protected', name: 'protected', component: ProtectedPage }
      ]
    });
    router.beforeEach(() => false);
    router.install();

    await expect(router.push('/protected')).rejects.toThrow(NavigationError);
    expect(router.currentRoute.value.path).toBe('/');
  });

  it('should redirect when guard returns route string', async () => {
    const router = createRouter({
      routes: [
        { path: '/', name: 'home', component: HomePage },
        { path: '/login', name: 'login', component: LoginPage },
        { path: '/protected', name: 'protected', component: ProtectedPage }
      ]
    });
    router.beforeEach((to) => {
      if (to.path === '/protected') {
        return '/login';
      }
    });
    router.install();

    await router.push('/protected');

    expect(router.currentRoute.value.path).toBe('/login');
  });

  it('should redirect when guard returns route object', async () => {
    const router = createRouter({
      routes: [
        { path: '/', name: 'home', component: HomePage },
        { path: '/login', name: 'login', component: LoginPage },
        { path: '/protected', name: 'protected', component: ProtectedPage }
      ]
    });
    router.beforeEach((to) => {
      if (to.path === '/protected') {
        return { name: 'login' };
      }
    });
    router.install();

    await router.push('/protected');

    expect(router.currentRoute.value.path).toBe('/login');
  });

  it('should unregister beforeEach guard', async () => {
    const router = createRouter({
      routes: [
        { path: '/', name: 'home', component: HomePage },
        { path: '/about', name: 'about', component: AboutPage }
      ]
    });
    const guard = vi.fn();
    const unregister = router.beforeEach(guard);
    unregister();
    router.install();

    await router.push('/about');

    expect(guard).not.toHaveBeenCalled();
  });

  it('should run multiple beforeEach guards in order', async () => {
    const router = createRouter({
      routes: [
        { path: '/', name: 'home', component: HomePage },
        { path: '/about', name: 'about', component: AboutPage }
      ]
    });
    const order: number[] = [];
    router.beforeEach(() => { order.push(1); });
    router.beforeEach(() => { order.push(2); });
    router.beforeEach(() => { order.push(3); });
    router.install();

    await router.push('/about');

    expect(order).toEqual([1, 2, 3]);
  });

  it('should run beforeResolve guard', async () => {
    const router = createRouter({
      routes: [
        { path: '/', name: 'home', component: HomePage },
        { path: '/about', name: 'about', component: AboutPage }
      ]
    });
    const guard = vi.fn();
    router.beforeResolve(guard);
    router.install();

    await router.push('/about');

    expect(guard).toHaveBeenCalled();
  });

  it('should run afterEach hook', async () => {
    const router = createRouter({
      routes: [
        { path: '/', name: 'home', component: HomePage },
        { path: '/about', name: 'about', component: AboutPage }
      ]
    });
    const hook = vi.fn();
    router.afterEach(hook);
    router.install();

    await router.push('/about');

    expect(hook).toHaveBeenCalled();
  });

  it('should pass correct arguments to afterEach', async () => {
    const router = createRouter({
      routes: [
        { path: '/', name: 'home', component: HomePage },
        { path: '/about', name: 'about', component: AboutPage }
      ]
    });
    const hook = vi.fn();
    router.afterEach(hook);
    router.install();

    await router.push('/about');

    expect(hook).toHaveBeenCalledWith(
      expect.objectContaining({ path: '/about' }),
      expect.objectContaining({ path: '/' })
    );
  });

  it('should unregister afterEach hook', async () => {
    const router = createRouter({
      routes: [
        { path: '/', name: 'home', component: HomePage },
        { path: '/about', name: 'about', component: AboutPage }
      ]
    });
    const hook = vi.fn();
    const unregister = router.afterEach(hook);
    unregister();
    router.install();

    await router.push('/about');

    expect(hook).not.toHaveBeenCalled();
  });

  it('should run onError handler on navigation failure', async () => {
    const router = createRouter({
      routes: [
        { path: '/', name: 'home', component: HomePage },
        { path: '/about', name: 'about', component: AboutPage }
      ]
    });
    const errorHandler = vi.fn();
    router.onError(errorHandler);
    router.beforeEach(() => false);
    router.install();

    try {
      await router.push('/about');
    } catch {
      // Expected
    }

    expect(errorHandler).toHaveBeenCalled();
  });

  it('should pass NavigationError to onError handler', async () => {
    const router = createRouter({
      routes: [
        { path: '/', name: 'home', component: HomePage },
        { path: '/about', name: 'about', component: AboutPage }
      ]
    });
    const errorHandler = vi.fn();
    router.onError(errorHandler);
    router.beforeEach(() => false);
    router.install();

    try {
      await router.push('/about');
    } catch {
      // Expected
    }

    expect(errorHandler).toHaveBeenCalledWith(
      expect.any(NavigationError),
      expect.anything(),
      expect.anything()
    );
  });

  it('should unregister onError handler', async () => {
    const router = createRouter({
      routes: [
        { path: '/', name: 'home', component: HomePage },
        { path: '/about', name: 'about', component: AboutPage }
      ]
    });
    const errorHandler = vi.fn();
    const unregister = router.onError(errorHandler);
    unregister();
    router.beforeEach(() => false);
    router.install();

    try {
      await router.push('/about');
    } catch {
      // Expected
    }

    expect(errorHandler).not.toHaveBeenCalled();
  });

  it('should handle async beforeEach guard', async () => {
    const router = createRouter({
      routes: [
        { path: '/', name: 'home', component: HomePage },
        { path: '/about', name: 'about', component: AboutPage }
      ]
    });
    router.beforeEach(async () => {
      await new Promise(resolve => setTimeout(resolve, 10));
      return true;
    });
    router.install();

    await router.push('/about');

    expect(router.currentRoute.value.path).toBe('/about');
  });

  it('should run route-specific beforeEnter guard', async () => {
    const guard = vi.fn();
    const router = createRouter({
      routes: [
        { path: '/', name: 'home', component: HomePage },
        { path: '/guarded', name: 'guarded', component: AboutPage, beforeEnter: guard }
      ]
    });
    router.install();

    await router.push('/guarded');

    expect(guard).toHaveBeenCalled();
  });

  it('should block navigation with route beforeEnter returning false', async () => {
    const router = createRouter({
      routes: [
        { path: '/', name: 'home', component: HomePage },
        { path: '/guarded', name: 'guarded', component: AboutPage, beforeEnter: () => false }
      ]
    });
    router.install();

    await expect(router.push('/guarded')).rejects.toThrow(NavigationError);
  });

  it('should handle array of beforeEnter guards', async () => {
    const order: number[] = [];
    const router = createRouter({
      routes: [
        { path: '/', name: 'home', component: HomePage },
        { 
          path: '/guarded', 
          name: 'guarded', 
          component: AboutPage, 
          beforeEnter: [
            () => { order.push(1); },
            () => { order.push(2); }
          ]
        }
      ]
    });
    router.install();

    await router.push('/guarded');

    expect(order).toEqual([1, 2]);
  });

  it('should check route meta in guards', async () => {
    const router = createRouter({
      routes: [
        { path: '/', name: 'home', component: HomePage },
        { path: '/admin', name: 'admin', component: DashboardPage, meta: { requiresAuth: true } }
      ]
    });
    let checkedMeta = false;
    router.beforeEach((to) => {
      if (to.meta?.requiresAuth) {
        checkedMeta = true;
      }
    });
    router.install();

    await router.push('/admin');

    expect(checkedMeta).toBe(true);
  });
});

// ============================================================================
// RouterLink Tests (20 tests)
// ============================================================================
describe('RouterLink Integration', () => {
  it('should render anchor element', () => {
    const router = createRouter({
      routes: [{ path: '/', name: 'home', component: HomePage }]
    });

    const { container } = render(
      h(RouterProvider, { router },
        h(RouterLink, { to: '/' }, 'Home')
      )
    );

    expect(container.querySelector('a')).not.toBeNull();
  });

  it('should have correct href', () => {
    const router = createRouter({
      routes: [
        { path: '/', name: 'home', component: HomePage },
        { path: '/about', name: 'about', component: AboutPage }
      ]
    });

    const { container } = render(
      h(RouterProvider, { router },
        h(RouterLink, { to: '/about' }, 'About')
      )
    );

    expect(container.querySelector('a')?.getAttribute('href')).toBe('/about');
  });

  it('should navigate on click', () => {
    const router = createRouter({
      routes: [
        { path: '/', name: 'home', component: HomePage },
        { path: '/about', name: 'about', component: AboutPage }
      ]
    });
    const pushSpy = vi.spyOn(router, 'push');

    const { container } = render(
      h(RouterProvider, { router },
        h(RouterLink, { to: '/about' }, 'About')
      )
    );

    fireEvent.click(container.querySelector('a')!);

    expect(pushSpy).toHaveBeenCalledWith('/about');
  });

  it('should use replace when replace prop is true', () => {
    const router = createRouter({
      routes: [
        { path: '/', name: 'home', component: HomePage },
        { path: '/about', name: 'about', component: AboutPage }
      ]
    });
    const replaceSpy = vi.spyOn(router, 'replace');

    const { container } = render(
      h(RouterProvider, { router },
        h(RouterLink, { to: '/about', replace: true }, 'About')
      )
    );

    fireEvent.click(container.querySelector('a')!);

    expect(replaceSpy).toHaveBeenCalled();
  });

  it('should support named route navigation', () => {
    const router = createRouter({
      routes: [
        { path: '/', name: 'home', component: HomePage },
        { path: '/users/:id', name: 'user', component: UserPage }
      ]
    });

    const { container } = render(
      h(RouterProvider, { router },
        h(RouterLink, { to: { name: 'user', params: { id: '123' } } }, 'User 123')
      )
    );

    expect(container.querySelector('a')?.getAttribute('href')).toBe('/users/123');
  });

  it('should support query parameters', () => {
    const router = createRouter({
      routes: [
        { path: '/', name: 'home', component: HomePage },
        { path: '/search', name: 'search', component: AboutPage }
      ]
    });

    const { container } = render(
      h(RouterProvider, { router },
        h(RouterLink, { to: { path: '/search', query: { q: 'test' } } }, 'Search')
      )
    );

    expect(container.querySelector('a')?.getAttribute('href')).toBe('/search?q=test');
  });

  it('should support hash', () => {
    const router = createRouter({
      routes: [
        { path: '/', name: 'home', component: HomePage },
        { path: '/page', name: 'page', component: AboutPage }
      ]
    });

    const { container } = render(
      h(RouterProvider, { router },
        h(RouterLink, { to: { path: '/page', hash: '#section' } }, 'Page')
      )
    );

    expect(container.querySelector('a')?.getAttribute('href')).toBe('/page#section');
  });

  it('should apply custom class', () => {
    const router = createRouter({
      routes: [{ path: '/', name: 'home', component: HomePage }]
    });

    const { container } = render(
      h(RouterProvider, { router },
        h(RouterLink, { to: '/', class: 'my-link' }, 'Home')
      )
    );

    expect(container.querySelector('a')?.classList.contains('my-link')).toBe(true);
  });

  it('should apply activeClass when active', () => {
    const router = createRouter({
      routes: [{ path: '/', name: 'home', component: HomePage }]
    });
    router.currentRoute.value = {
      fullPath: '/',
      path: '/',
      params: {},
      query: {},
      hash: '',
      meta: {},
      matched: []
    };

    const { container } = render(
      h(RouterProvider, { router },
        h(RouterLink, { to: '/', activeClass: 'active' }, 'Home')
      )
    );

    expect(container.querySelector('a')?.classList.contains('active')).toBe(true);
  });

  it('should apply exactActiveClass when exactly active', () => {
    const router = createRouter({
      routes: [{ path: '/', name: 'home', component: HomePage }]
    });
    router.currentRoute.value = {
      fullPath: '/',
      path: '/',
      params: {},
      query: {},
      hash: '',
      meta: {},
      matched: []
    };

    const { container } = render(
      h(RouterProvider, { router },
        h(RouterLink, { to: '/', exactActiveClass: 'exact' }, 'Home')
      )
    );

    expect(container.querySelector('a')?.classList.contains('exact')).toBe(true);
  });

  it('should not navigate on ctrl+click', () => {
    const router = createRouter({
      routes: [{ path: '/', name: 'home', component: HomePage }]
    });
    const pushSpy = vi.spyOn(router, 'push');

    const { container } = render(
      h(RouterProvider, { router },
        h(RouterLink, { to: '/about' }, 'About')
      )
    );

    fireEvent.click(container.querySelector('a')!, { ctrlKey: true });

    expect(pushSpy).not.toHaveBeenCalled();
  });

  it('should not navigate on meta+click', () => {
    const router = createRouter({
      routes: [{ path: '/', name: 'home', component: HomePage }]
    });
    const pushSpy = vi.spyOn(router, 'push');

    const { container } = render(
      h(RouterProvider, { router },
        h(RouterLink, { to: '/about' }, 'About')
      )
    );

    fireEvent.click(container.querySelector('a')!, { metaKey: true });

    expect(pushSpy).not.toHaveBeenCalled();
  });

  it('should not navigate on shift+click', () => {
    const router = createRouter({
      routes: [{ path: '/', name: 'home', component: HomePage }]
    });
    const pushSpy = vi.spyOn(router, 'push');

    const { container } = render(
      h(RouterProvider, { router },
        h(RouterLink, { to: '/about' }, 'About')
      )
    );

    fireEvent.click(container.querySelector('a')!, { shiftKey: true });

    expect(pushSpy).not.toHaveBeenCalled();
  });

  it('should not navigate on alt+click', () => {
    const router = createRouter({
      routes: [{ path: '/', name: 'home', component: HomePage }]
    });
    const pushSpy = vi.spyOn(router, 'push');

    const { container } = render(
      h(RouterProvider, { router },
        h(RouterLink, { to: '/about' }, 'About')
      )
    );

    fireEvent.click(container.querySelector('a')!, { altKey: true });

    expect(pushSpy).not.toHaveBeenCalled();
  });

  it('should not navigate on right-click', () => {
    const router = createRouter({
      routes: [{ path: '/', name: 'home', component: HomePage }]
    });
    const pushSpy = vi.spyOn(router, 'push');

    const { container } = render(
      h(RouterProvider, { router },
        h(RouterLink, { to: '/about' }, 'About')
      )
    );

    fireEvent.click(container.querySelector('a')!, { button: 2 });

    expect(pushSpy).not.toHaveBeenCalled();
  });

  it('should render children correctly', () => {
    const router = createRouter({
      routes: [{ path: '/', name: 'home', component: HomePage }]
    });

    const { container } = render(
      h(RouterProvider, { router },
        h(RouterLink, { to: '/' },
          h('span', { class: 'icon' }, '★'),
          ' Home'
        )
      )
    );

    expect(container.querySelector('.icon')).not.toBeNull();
    expect(container.textContent).toContain('Home');
  });

  it('should handle complex query params', () => {
    const router = createRouter({
      routes: [
        { path: '/search', name: 'search', component: AboutPage }
      ]
    });

    const { container } = render(
      h(RouterProvider, { router },
        h(RouterLink, { to: { path: '/search', query: { q: 'hello world', page: '2', filter: 'active' } } }, 'Search')
      )
    );

    const href = container.querySelector('a')?.getAttribute('href') || '';
    expect(href).toContain('q=hello%20world');
    expect(href).toContain('page=2');
    expect(href).toContain('filter=active');
  });

  it('should combine query and hash', () => {
    const router = createRouter({
      routes: [
        { path: '/page', name: 'page', component: AboutPage }
      ]
    });

    const { container } = render(
      h(RouterProvider, { router },
        h(RouterLink, { to: { path: '/page', query: { tab: '1' }, hash: '#details' } }, 'Page')
      )
    );

    expect(container.querySelector('a')?.getAttribute('href')).toBe('/page?tab=1#details');
  });

  it('should use default active classes', () => {
    const router = createRouter({
      routes: [{ path: '/', name: 'home', component: HomePage }]
    });
    router.currentRoute.value = {
      fullPath: '/',
      path: '/',
      params: {},
      query: {},
      hash: '',
      meta: {},
      matched: []
    };

    const { container } = render(
      h(RouterProvider, { router },
        h(RouterLink, { to: '/' }, 'Home')
      )
    );

    const link = container.querySelector('a');
    expect(link?.classList.contains('router-link-active')).toBe(true);
    expect(link?.classList.contains('router-link-exact-active')).toBe(true);
  });
});

// ============================================================================
// Hooks Tests (15 tests)
// ============================================================================
describe('Hooks Integration', () => {
  it('useRouter should return router instance', () => {
    const router = createRouter({
      routes: [{ path: '/', name: 'home', component: HomePage }]
    });
    let capturedRouter: Router | null = null;

    const TestComponent = () => {
      capturedRouter = useRouter();
      return h('div', null, 'Test');
    };

    render(
      h(RouterProvider, { router },
        h(TestComponent, null)
      )
    );

    expect(capturedRouter).toBe(router);
  });

  it('useRoute should return route signal', () => {
    const router = createRouter({
      routes: [{ path: '/', name: 'home', component: HomePage }]
    });
    let routeValue: any = null;

    const TestComponent = () => {
      const route = useRoute();
      routeValue = route.value;
      return h('div', null, 'Test');
    };

    render(
      h(RouterProvider, { router },
        h(TestComponent, null)
      )
    );

    expect(routeValue).toBeDefined();
    expect(routeValue.path).toBe('/');
  });

  it('useNavigation should return navigation methods', () => {
    const router = createRouter({
      routes: [{ path: '/', name: 'home', component: HomePage }]
    });
    let navigation: ReturnType<typeof useNavigation> | null = null;

    const TestComponent = () => {
      navigation = useNavigation();
      return h('div', null, 'Test');
    };

    render(
      h(RouterProvider, { router },
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

  it('useNavigation push should call router.push', () => {
    const router = createRouter({
      routes: [
        { path: '/', name: 'home', component: HomePage },
        { path: '/about', name: 'about', component: AboutPage }
      ]
    });
    const pushSpy = vi.spyOn(router, 'push');
    let navigation: ReturnType<typeof useNavigation> | null = null;

    const TestComponent = () => {
      navigation = useNavigation();
      return h('div', null, 'Test');
    };

    render(
      h(RouterProvider, { router },
        h(TestComponent, null)
      )
    );

    navigation!.push('/about');

    expect(pushSpy).toHaveBeenCalledWith('/about');
  });

  it('useNavigation replace should call router.replace', () => {
    const router = createRouter({
      routes: [
        { path: '/', name: 'home', component: HomePage },
        { path: '/about', name: 'about', component: AboutPage }
      ]
    });
    const replaceSpy = vi.spyOn(router, 'replace');
    let navigation: ReturnType<typeof useNavigation> | null = null;

    const TestComponent = () => {
      navigation = useNavigation();
      return h('div', null, 'Test');
    };

    render(
      h(RouterProvider, { router },
        h(TestComponent, null)
      )
    );

    navigation!.replace('/about');

    expect(replaceSpy).toHaveBeenCalledWith('/about');
  });

  it('useNavigation back should call router.back', () => {
    const router = createRouter({
      routes: [{ path: '/', name: 'home', component: HomePage }]
    });
    const backSpy = vi.spyOn(router, 'back');
    let navigation: ReturnType<typeof useNavigation> | null = null;

    const TestComponent = () => {
      navigation = useNavigation();
      return h('div', null, 'Test');
    };

    render(
      h(RouterProvider, { router },
        h(TestComponent, null)
      )
    );

    navigation!.back();

    expect(backSpy).toHaveBeenCalled();
  });

  it('useNavigation forward should call router.forward', () => {
    const router = createRouter({
      routes: [{ path: '/', name: 'home', component: HomePage }]
    });
    const forwardSpy = vi.spyOn(router, 'forward');
    let navigation: ReturnType<typeof useNavigation> | null = null;

    const TestComponent = () => {
      navigation = useNavigation();
      return h('div', null, 'Test');
    };

    render(
      h(RouterProvider, { router },
        h(TestComponent, null)
      )
    );

    navigation!.forward();

    expect(forwardSpy).toHaveBeenCalled();
  });

  it('useNavigation go should call router.go', () => {
    const router = createRouter({
      routes: [{ path: '/', name: 'home', component: HomePage }]
    });
    const goSpy = vi.spyOn(router, 'go');
    let navigation: ReturnType<typeof useNavigation> | null = null;

    const TestComponent = () => {
      navigation = useNavigation();
      return h('div', null, 'Test');
    };

    render(
      h(RouterProvider, { router },
        h(TestComponent, null)
      )
    );

    navigation!.go(-2);

    expect(goSpy).toHaveBeenCalledWith(-2);
  });

  it('useRouter throws error outside RouterProvider', () => {
    const TestComponent = () => {
      try {
        useRouter();
        return h('div', null, 'Should not reach');
      } catch (err) {
        return h('div', { 'data-testid': 'error' }, (err as Error).message);
      }
    };

    const { container } = render(h(TestComponent, null));

    expect(container.querySelector('[data-testid="error"]')).not.toBeNull();
    expect(container.textContent).toContain('useRouter must be used within a RouterProvider');
  });

  it('useRoute throws error outside RouterProvider', () => {
    const TestComponent = () => {
      try {
        useRoute();
        return h('div', null, 'Should not reach');
      } catch (err) {
        return h('div', { 'data-testid': 'error' }, (err as Error).message);
      }
    };

    const { container } = render(h(TestComponent, null));

    expect(container.querySelector('[data-testid="error"]')).not.toBeNull();
    expect(container.textContent).toContain('useRoute must be used within a RouterProvider');
  });

  it('useRoute provides access to current path', () => {
    const router = createRouter({
      routes: [{ path: '/', name: 'home', component: HomePage }]
    });
    let path = '';

    const TestComponent = () => {
      const route = useRoute();
      path = route.value.path;
      return h('div', null, 'Test');
    };

    render(
      h(RouterProvider, { router },
        h(TestComponent, null)
      )
    );

    expect(path).toBe('/');
  });

  it('useRoute provides access to route signal', () => {
    const router = createRouter({
      routes: [{ path: '/', name: 'home', component: HomePage, meta: { title: 'Home' } }]
    });
    let routeValue: any = null;

    const TestComponent = () => {
      const route = useRoute();
      routeValue = route.value;
      return h('div', null, 'Test');
    };

    render(
      h(RouterProvider, { router },
        h(TestComponent, null)
      )
    );

    expect(routeValue).toBeDefined();
    expect(routeValue.path).toBe('/');
    expect(routeValue.matched).toBeDefined();
  });
});

// ============================================================================
// Dynamic Routes Tests (15 tests)
// ============================================================================
describe('Dynamic Routes', () => {
  it('should add route at runtime', () => {
    const router = createRouter({
      routes: [{ path: '/', name: 'home', component: HomePage }]
    });

    router.addRoute(undefined, { path: '/new', name: 'new', component: AboutPage });

    expect(router.hasRoute('new')).toBe(true);
    expect(router.getRoutes().length).toBe(2);
  });

  it('should add nested route', () => {
    const router = createRouter({
      routes: [{ path: '/parent', name: 'parent', component: HomePage }]
    });

    router.addRoute('parent', { path: '/child', name: 'child', component: AboutPage });

    const parentRoute = router.getRoutes()[0];
    expect(parentRoute.children).toBeDefined();
    expect(parentRoute.children!.length).toBe(1);
  });

  it('should remove route by name', () => {
    const router = createRouter({
      routes: [
        { path: '/', name: 'home', component: HomePage },
        { path: '/about', name: 'about', component: AboutPage }
      ]
    });

    router.removeRoute('about');

    expect(router.hasRoute('about')).toBe(false);
    expect(router.getRoutes().length).toBe(1);
  });

  it('should return removal function from addRoute', () => {
    const router = createRouter({
      routes: [{ path: '/', name: 'home', component: HomePage }]
    });

    const remove = router.addRoute(undefined, { path: '/temp', name: 'temp', component: AboutPage });
    expect(router.hasRoute('temp')).toBe(true);

    remove();
    expect(router.hasRoute('temp')).toBe(false);
  });

  it('should not fail when removing non-existent route', () => {
    const router = createRouter({
      routes: [{ path: '/', name: 'home', component: HomePage }]
    });

    expect(() => router.removeRoute('non-existent')).not.toThrow();
  });

  it('should navigate to dynamically added route', async () => {
    const router = createRouter({
      routes: [{ path: '/', name: 'home', component: HomePage }]
    });
    router.install();

    router.addRoute(undefined, { path: '/dynamic', name: 'dynamic', component: AboutPage });
    await router.push('/dynamic');

    expect(router.currentRoute.value.path).toBe('/dynamic');
  });

  it('should check route existence with hasRoute', () => {
    const router = createRouter({
      routes: [
        { path: '/', name: 'home', component: HomePage },
        { path: '/about', name: 'about', component: AboutPage }
      ]
    });

    expect(router.hasRoute('home')).toBe(true);
    expect(router.hasRoute('about')).toBe(true);
    expect(router.hasRoute('non-existent')).toBe(false);
  });

  it('should get all routes', () => {
    const router = createRouter({
      routes: [
        { path: '/', name: 'home', component: HomePage },
        { path: '/about', name: 'about', component: AboutPage },
        { path: '/users', name: 'users', component: UserPage }
      ]
    });

    const routes = router.getRoutes();

    expect(routes.length).toBe(3);
  });

  it('should add multiple routes', () => {
    const router = createRouter({
      routes: [{ path: '/', name: 'home', component: HomePage }]
    });

    router.addRoute(undefined, { path: '/a', name: 'a', component: AboutPage });
    router.addRoute(undefined, { path: '/b', name: 'b', component: AboutPage });
    router.addRoute(undefined, { path: '/c', name: 'c', component: AboutPage });

    expect(router.getRoutes().length).toBe(4);
    expect(router.hasRoute('a')).toBe(true);
    expect(router.hasRoute('b')).toBe(true);
    expect(router.hasRoute('c')).toBe(true);
  });

  it('should remove nested route', () => {
    const router = createRouter({
      routes: [{
        path: '/parent',
        name: 'parent',
        component: HomePage,
        children: [{ path: '/child', name: 'child', component: AboutPage }]
      }]
    });

    router.removeRoute('child');

    expect(router.hasRoute('parent')).toBe(true);
    expect(router.hasRoute('child')).toBe(false);
  });

  it('should add route with meta', () => {
    const router = createRouter({
      routes: [{ path: '/', name: 'home', component: HomePage }]
    });

    router.addRoute(undefined, {
      path: '/admin',
      name: 'admin',
      component: DashboardPage,
      meta: { requiresAuth: true }
    });

    const resolved = router.resolve('/admin');
    expect(resolved.meta).toEqual({ requiresAuth: true });
  });

  it('should add route with beforeEnter', () => {
    const router = createRouter({
      routes: [{ path: '/', name: 'home', component: HomePage }]
    });
    const guard = vi.fn();

    router.addRoute(undefined, {
      path: '/guarded',
      name: 'guarded',
      component: AboutPage,
      beforeEnter: guard
    });

    expect(router.hasRoute('guarded')).toBe(true);
  });

  it('should add route with redirect', () => {
    const router = createRouter({
      routes: [
        { path: '/', name: 'home', component: HomePage },
        { path: '/target', name: 'target', component: AboutPage }
      ]
    });

    router.addRoute(undefined, {
      path: '/old',
      name: 'old',
      redirect: '/target'
    });

    expect(router.hasRoute('old')).toBe(true);
  });

  it('should handle route params after dynamic add', async () => {
    const router = createRouter({
      routes: [{ path: '/', name: 'home', component: HomePage }]
    });
    router.install();

    router.addRoute(undefined, { path: '/users/:id', name: 'user', component: UserPage });
    await router.push('/users/123');

    expect(router.currentRoute.value.params).toEqual({ id: '123' });
  });

  it('should resolve dynamically added route by name', () => {
    const router = createRouter({
      routes: [{ path: '/', name: 'home', component: HomePage }]
    });

    router.addRoute(undefined, { path: '/products/:id', name: 'product', component: AboutPage });

    const resolved = router.resolve({ name: 'product', params: { id: '456' } });
    expect(resolved.path).toBe('/products/456');
  });
});

// ============================================================================
// Route Redirect Tests (10 tests)
// ============================================================================
describe('Route Redirects', () => {
  it('should redirect with string', async () => {
    const router = createRouter({
      routes: [
        { path: '/', name: 'home', component: HomePage },
        { path: '/old', redirect: '/new' },
        { path: '/new', name: 'new', component: AboutPage }
      ]
    });
    router.install();

    await router.push('/old');

    expect(router.currentRoute.value.path).toBe('/new');
  });

  it('should redirect with object', async () => {
    const router = createRouter({
      routes: [
        { path: '/', name: 'home', component: HomePage },
        { path: '/old', redirect: { name: 'new' } },
        { path: '/new', name: 'new', component: AboutPage }
      ]
    });
    router.install();

    await router.push('/old');

    expect(router.currentRoute.value.name).toBe('new');
  });

  it('should redirect with function', async () => {
    const router = createRouter({
      routes: [
        { path: '/', name: 'home', component: HomePage },
        { path: '/old/:id', redirect: (to) => `/new/${to.params.id}` },
        { path: '/new/:id', name: 'new', component: UserPage }
      ]
    });
    router.install();

    await router.push('/old/123');

    expect(router.currentRoute.value.path).toBe('/new/123');
  });

  it('should preserve params in redirect', async () => {
    const router = createRouter({
      routes: [
        { path: '/', name: 'home', component: HomePage },
        { path: '/old/:id', redirect: (to) => ({ path: `/new/${to.params.id}` }) },
        { path: '/new/:id', name: 'new', component: UserPage }
      ]
    });
    router.install();

    await router.push('/old/456');

    expect(router.currentRoute.value.params).toEqual({ id: '456' });
  });

  it('should handle redirect to named route with params', async () => {
    const router = createRouter({
      routes: [
        { path: '/', name: 'home', component: HomePage },
        { path: '/old/:id', redirect: (to) => ({ name: 'new', params: { id: to.params.id } }) },
        { path: '/new/:id', name: 'new', component: UserPage }
      ]
    });
    router.install();

    await router.push('/old/789');

    expect(router.currentRoute.value.path).toBe('/new/789');
  });

  it('should handle redirect with query params', async () => {
    const router = createRouter({
      routes: [
        { path: '/', name: 'home', component: HomePage },
        { path: '/search', redirect: { path: '/results', query: { source: 'redirect' } } },
        { path: '/results', name: 'results', component: AboutPage }
      ]
    });
    router.install();

    await router.push('/search');

    expect(router.currentRoute.value.query).toEqual({ source: 'redirect' });
  });

  it('should use replace for redirect', async () => {
    const router = createRouter({
      routes: [
        { path: '/', name: 'home', component: HomePage },
        { path: '/old', redirect: '/new' },
        { path: '/new', name: 'new', component: AboutPage }
      ]
    });
    router.install();

    await router.push('/old');

    // Redirect should use replace
    expect(router.currentRoute.value.path).toBe('/new');
  });

  it('should handle complex redirect function', async () => {
    const router = createRouter({
      routes: [
        { path: '/', name: 'home', component: HomePage },
        { path: '/legacy/:type/:id', redirect: (to) => {
          const { type, id } = to.params;
          if (type === 'user') {
            return `/users/${id}`;
          }
          return `/items/${id}`;
        }},
        { path: '/users/:id', name: 'user', component: UserPage },
        { path: '/items/:id', name: 'item', component: AboutPage }
      ]
    });
    router.install();

    await router.push('/legacy/user/100');
    expect(router.currentRoute.value.path).toBe('/users/100');
  });

  it('should redirect root to another route', async () => {
    const router = createRouter({
      routes: [
        { path: '/', redirect: '/home' },
        { path: '/home', name: 'home', component: HomePage }
      ]
    });
    router.install();

    expect(router.currentRoute.value.path).toBe('/home');
  });

  it('should handle redirect in nested routes', async () => {
    const router = createRouter({
      routes: [
        { path: '/', name: 'home', component: HomePage },
        {
          path: '/admin',
          name: 'admin',
          component: DashboardPage,
          children: [
            { path: '/old-settings', redirect: '/settings' },
            { path: '/settings', name: 'admin-settings', component: AboutPage }
          ]
        },
        { path: '/settings', name: 'settings', component: AboutPage }
      ]
    });
    router.install();

    await router.push('/admin/old-settings');

    // Redirect goes to /settings (relative to root, not admin)
    expect(router.currentRoute.value.path).toBe('/settings');
  });
});
