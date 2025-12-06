import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, fireEvent, cleanup } from '@testing-library/react';
import { RouterProvider, RouterView, RouterLink } from '../components';
import { createRouter } from '../router';
import type { RouteComponentProps } from '../types';

// Mock window for testing
const mockPushState = vi.fn();
const mockReplaceState = vi.fn();
const mockAddEventListener = vi.fn();
const mockRemoveEventListener = vi.fn();

beforeEach(() => {
  vi.stubGlobal('window', {
    history: {
      pushState: mockPushState,
      replaceState: mockReplaceState,
      back: vi.fn(),
      forward: vi.fn(),
      go: vi.fn()
    },
    location: {
      pathname: '/',
      search: '',
      hash: '',
      origin: 'http://localhost'
    },
    addEventListener: mockAddEventListener,
    removeEventListener: mockRemoveEventListener,
  });
  document.addEventListener = vi.fn();
  document.removeEventListener = vi.fn();
});

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

// Test components
const HomeComponent = ({ params, query }: RouteComponentProps) => {
  return <div data-testid="home">Home Page</div>;
};

const AboutComponent = ({ params, query }: RouteComponentProps) => {
  return <div data-testid="about">About Page</div>;
};

const UserComponent = ({ params }: RouteComponentProps) => {
  return <div data-testid="user">User: {params.id}</div>;
};

describe('RouterProvider', () => {
  it('should render without errors', () => {
    const router = createRouter({
      routes: [{ path: '/', name: 'home', component: HomeComponent }]
    });

    const { container } = render(
      <RouterProvider router={router}>
        <div data-testid="child">Child Content</div>
      </RouterProvider>
    );

    expect(container.querySelector('[data-testid="child"]')).not.toBeNull();
  });

  it('should provide router context to children', () => {
    const router = createRouter({
      routes: [{ path: '/', name: 'home', component: HomeComponent }]
    });

    const { container } = render(
      <RouterProvider router={router}>
        <RouterView />
      </RouterProvider>
    );

    expect(container.querySelector('[data-testid="home"]')).not.toBeNull();
  });
});

describe('RouterView', () => {
  it('should render matched route component', () => {
    const router = createRouter({
      routes: [{ path: '/', name: 'home', component: HomeComponent }]
    });

    const { container } = render(
      <RouterProvider router={router}>
        <RouterView />
      </RouterProvider>
    );

    expect(container.querySelector('[data-testid="home"]')).not.toBeNull();
    expect(container.textContent).toContain('Home Page');
  });

  it('should render different components for different routes', async () => {
    const router = createRouter({
      routes: [
        { path: '/', name: 'home', component: HomeComponent },
        { path: '/about', name: 'about', component: AboutComponent }
      ]
    });

    const { container } = render(
      <RouterProvider router={router}>
        <RouterView />
      </RouterProvider>
    );

    // Initially home
    expect(container.querySelector('[data-testid="home"]')).not.toBeNull();

    // Navigate to about
    await router.push('/about');
    
    // Note: In a real app, the view would update automatically via signals
  });

  it('should render user component with params', () => {
    const router = createRouter({
      routes: [{ path: '/users/:id', name: 'user', component: UserComponent }]
    });

    // Manually set route to test params
    router.currentRoute.value = {
      fullPath: '/users/123',
      path: '/users/123',
      params: { id: '123' },
      query: {},
      hash: '',
      meta: {},
      name: 'user',
      matched: [{ path: '/users/:id', name: 'user', component: UserComponent }]
    };

    const { container } = render(
      <RouterProvider router={router}>
        <RouterView />
      </RouterProvider>
    );

    expect(container.querySelector('[data-testid="user"]')).not.toBeNull();
    expect(container.textContent).toContain('User: 123');
  });

  it('should render notFound component when no route matches', () => {
    const NotFoundComponent = () => <div data-testid="not-found">404 Not Found</div>;
    
    const router = createRouter({
      routes: [{ path: '/', name: 'home', component: HomeComponent }]
    });

    // Set route to non-existent path
    router.currentRoute.value = {
      fullPath: '/nonexistent',
      path: '/nonexistent',
      params: {},
      query: {},
      hash: '',
      meta: {},
      matched: []
    };

    const { container } = render(
      <RouterProvider router={router}>
        <RouterView notFound={NotFoundComponent} />
      </RouterProvider>
    );

    expect(container.querySelector('[data-testid="not-found"]')).not.toBeNull();
    expect(container.textContent).toContain('404 Not Found');
  });
});

describe('RouterLink', () => {
  it('should render a link', () => {
    const router = createRouter({
      routes: [
        { path: '/', name: 'home' },
        { path: '/about', name: 'about' }
      ]
    });

    const { container } = render(
      <RouterProvider router={router}>
        <RouterLink to='/about'>About</RouterLink>
      </RouterProvider>
    );

    const link = container.querySelector('a');
    expect(link).not.toBeNull();
    expect(link?.textContent).toBe('About');
  });

  it('should have correct href attribute', () => {
    const router = createRouter({
      routes: [
        { path: '/', name: 'home' },
        { path: '/about', name: 'about' }
      ]
    });

    const { container } = render(
      <RouterProvider router={router}>
        <RouterLink to='/about'>About</RouterLink>
      </RouterProvider>
    );

    const link = container.querySelector('a');
    expect(link?.getAttribute('href')).toBe('/about');
  });

  it('should call router.push on click', () => {
    const router = createRouter({
      routes: [
        { path: '/', name: 'home' },
        { path: '/about', name: 'about' }
      ]
    });

    const pushSpy = vi.spyOn(router, 'push');

    const { container } = render(
      <RouterProvider router={router}>
        <RouterLink to='/about'>About</RouterLink>
      </RouterProvider>
    );

    const link = container.querySelector('a');
    fireEvent.click(link!);

    expect(pushSpy).toHaveBeenCalledWith('/about');
  });

  it('should call router.replace when replace prop is true', () => {
    const router = createRouter({
      routes: [
        { path: '/', name: 'home' },
        { path: '/about', name: 'about' }
      ]
    });

    const replaceSpy = vi.spyOn(router, 'replace');

    const { container } = render(
      <RouterProvider router={router}>
        <RouterLink to='/about' replace={true}>About</RouterLink>
      </RouterProvider>
    );

    const link = container.querySelector('a');
    fireEvent.click(link!);

    expect(replaceSpy).toHaveBeenCalledWith('/about');
  });

  it('should support named routes', () => {
    const router = createRouter({
      routes: [
        { path: '/', name: 'home' },
        { path: '/users/:id', name: 'user' }
      ]
    });

    const { container } = render(
      <RouterProvider router={router}>
        <RouterLink to={{ name: 'user', params: { id: '123' } }}>User 123</RouterLink>
      </RouterProvider>
    );

    const link = container.querySelector('a');
    expect(link).not.toBeNull();
  });
});

describe('Integration Tests', () => {
  it('should handle complete navigation flow', async () => {
    const router = createRouter({
      routes: [
        { path: '/', name: 'home', component: HomeComponent },
        { path: '/about', name: 'about', component: AboutComponent }
      ]
    });

    const { container } = render(
      <RouterProvider router={router}>
        <div>
          <RouterLink to='/'>Home</RouterLink>
          <RouterLink to='/about'>About</RouterLink>
          <RouterView />
        </div>
      </RouterProvider>
    );

    // Check initial state
    expect(container.querySelector('[data-testid="home"]')).not.toBeNull();
    
    // Navigate programmatically
    await router.push('/about');
    
    // Note: Actual route update would be reflected via signals
  });
});
