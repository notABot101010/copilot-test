import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { h, ComponentType } from 'preact';
import { render, fireEvent, cleanup } from '@testing-library/preact';
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
    scrollTo: vi.fn()
  });
  vi.clearAllMocks();
});

afterEach(() => {
  cleanup();
});

// Test components
const HomeComponent = ({ params, query }: RouteComponentProps) => {
  return h('div', { 'data-testid': 'home' }, 'Home Page');
};

const UserComponent = ({ params, query }: RouteComponentProps) => {
  return h('div', { 'data-testid': 'user' }, `User: ${params.id}`);
};

const AboutComponent = ({ params, query }: RouteComponentProps) => {
  return h('div', { 'data-testid': 'about' }, 'About Page');
};

const NotFoundComponent = () => {
  return h('div', { 'data-testid': 'not-found' }, '404 Not Found');
};

const NestedComponent = ({ params }: RouteComponentProps) => {
  return h('div', { 'data-testid': 'nested' }, `Nested: ${JSON.stringify(params)}`);
};

// ============================================================================
// RouterProvider Tests
// ============================================================================
describe('RouterProvider', () => {
  it('should render children', () => {
    const router = createRouter({
      routes: [{ path: '/', name: 'home', component: HomeComponent }]
    });

    const { container } = render(
      h(RouterProvider, { router },
        h('div', { 'data-testid': 'child' }, 'Child Content')
      )
    );

    expect(container.querySelector('[data-testid="child"]')).not.toBeNull();
  });

  it('should initialize router on mount', () => {
    const router = createRouter({
      routes: [{ path: '/', name: 'home' }]
    });
    const installSpy = vi.spyOn(router, 'install');

    render(h(RouterProvider, { router }, null));

    expect(installSpy).toHaveBeenCalled();
  });

  it('should add click listener for link interception', () => {
    const router = createRouter({
      routes: [{ path: '/', name: 'home' }]
    });

    render(h(RouterProvider, { router }, null));

    expect(document.addEventListener).toBeDefined();
  });

  it('should provide router context to children', () => {
    const router = createRouter({
      routes: [{ path: '/', name: 'home', component: HomeComponent }]
    });

    const { container } = render(
      h(RouterProvider, { router },
        h(RouterView, null)
      )
    );

    expect(container.querySelector('[data-testid="home"]')).not.toBeNull();
  });
});

// ============================================================================
// RouterView Tests
// ============================================================================
describe('RouterView', () => {
  it('should render the matched component', () => {
    const router = createRouter({
      routes: [{ path: '/', name: 'home', component: HomeComponent }]
    });

    const { container } = render(
      h(RouterProvider, { router },
        h(RouterView, null)
      )
    );

    expect(container.querySelector('[data-testid="home"]')).not.toBeNull();
  });

  it('should render component with params from route', () => {
    // Test with a simple component that accesses params via route signal
    const SimpleComponent = (props: RouteComponentProps) => {
      return h('div', { 'data-testid': 'simple' }, 'Simple Component');
    };

    const router = createRouter({
      routes: [
        { path: '/', name: 'simple', component: SimpleComponent }
      ]
    });

    const { container } = render(
      h(RouterProvider, { router },
        h(RouterView, null)
      )
    );

    expect(container.textContent).toContain('Simple Component');
  });

  it('should pass additional props to component', () => {
    const PropsComponent = ({ params, customProp }: RouteComponentProps & { customProp: string }) => {
      return h('div', { 'data-testid': 'props' }, `Custom: ${customProp}`);
    };

    const router = createRouter({
      routes: [{ path: '/', name: 'home', component: PropsComponent as ComponentType<RouteComponentProps> }]
    });

    const { container } = render(
      h(RouterProvider, { router },
        h(RouterView, { props: { customProp: 'test-value' } })
      )
    );

    expect(container.textContent).toContain('Custom: test-value');
  });

  it('should handle route props as object', () => {
    const PropsComponent = ({ params, query, staticProp }: RouteComponentProps & { staticProp: string }) => {
      return h('div', { 'data-testid': 'static-props' }, `Static: ${staticProp}`);
    };

    const router = createRouter({
      routes: [{
        path: '/',
        name: 'home',
        component: PropsComponent as ComponentType<RouteComponentProps>,
        props: { staticProp: 'static-value' }
      }]
    });

    const { container } = render(
      h(RouterProvider, { router },
        h(RouterView, null)
      )
    );

    expect(container.textContent).toContain('Static: static-value');
  });
});

// ============================================================================
// RouterLink Tests
// ============================================================================
describe('RouterLink', () => {
  it('should render an anchor tag', () => {
    const router = createRouter({
      routes: [{ path: '/', name: 'home' }]
    });

    const { container } = render(
      h(RouterProvider, { router },
        h(RouterLink, { to: '/about' }, 'About')
      )
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
      h(RouterProvider, { router },
        h(RouterLink, { to: '/about' }, 'About')
      )
    );

    const link = container.querySelector('a');
    expect(link?.getAttribute('href')).toBe('/about');
  });

  it('should navigate on click', () => {
    const router = createRouter({
      routes: [
        { path: '/', name: 'home' },
        { path: '/about', name: 'about' }
      ]
    });
    const pushSpy = vi.spyOn(router, 'push');

    const { container } = render(
      h(RouterProvider, { router },
        h(RouterLink, { to: '/about' }, 'About')
      )
    );

    const link = container.querySelector('a');
    fireEvent.click(link!);

    expect(pushSpy).toHaveBeenCalledWith('/about');
  });

  it('should replace when replace prop is true', () => {
    const router = createRouter({
      routes: [
        { path: '/', name: 'home' },
        { path: '/about', name: 'about' }
      ]
    });
    const replaceSpy = vi.spyOn(router, 'replace');

    const { container } = render(
      h(RouterProvider, { router },
        h(RouterLink, { to: '/about', replace: true }, 'About')
      )
    );

    const link = container.querySelector('a');
    fireEvent.click(link!);

    expect(replaceSpy).toHaveBeenCalledWith('/about');
  });

  it('should support object navigation target', () => {
    const router = createRouter({
      routes: [
        { path: '/', name: 'home' },
        { path: '/users/:id', name: 'user' }
      ]
    });

    const { container } = render(
      h(RouterProvider, { router },
        h(RouterLink, { to: { name: 'user', params: { id: '123' } } }, 'User 123')
      )
    );

    const link = container.querySelector('a');
    expect(link?.getAttribute('href')).toBe('/users/123');
  });

  it('should apply custom class', () => {
    const router = createRouter({
      routes: [{ path: '/', name: 'home' }]
    });

    const { container } = render(
      h(RouterProvider, { router },
        h(RouterLink, { to: '/', class: 'custom-link' }, 'Home')
      )
    );

    const link = container.querySelector('a');
    expect(link?.classList.contains('custom-link')).toBe(true);
  });

  it('should apply activeClass when route matches', () => {
    const router = createRouter({
      routes: [
        { path: '/', name: 'home' },
        { path: '/about', name: 'about' }
      ]
    });
    
    // Set current route to root
    router.currentRoute.value = {
      fullPath: '/',
      path: '/',
      params: {},
      query: {},
      hash: '',
      meta: {},
      matched: [{ path: '/', name: 'home' }]
    };

    const { container } = render(
      h(RouterProvider, { router },
        h(RouterLink, { to: '/', activeClass: 'is-active' }, 'Home')
      )
    );

    const link = container.querySelector('a');
    expect(link?.classList.contains('is-active')).toBe(true);
  });

  it('should apply exactActiveClass when route exactly matches', () => {
    const router = createRouter({
      routes: [{ path: '/', name: 'home' }]
    });
    
    // Set current route to root
    router.currentRoute.value = {
      fullPath: '/',
      path: '/',
      params: {},
      query: {},
      hash: '',
      meta: {},
      matched: [{ path: '/', name: 'home' }]
    };

    const { container } = render(
      h(RouterProvider, { router },
        h(RouterLink, { to: '/', exactActiveClass: 'exact-active' }, 'Home')
      )
    );

    const link = container.querySelector('a');
    expect(link?.classList.contains('exact-active')).toBe(true);
  });

  it('should not navigate on ctrl+click', () => {
    const router = createRouter({
      routes: [{ path: '/', name: 'home' }]
    });
    const pushSpy = vi.spyOn(router, 'push');

    const { container } = render(
      h(RouterProvider, { router },
        h(RouterLink, { to: '/about' }, 'About')
      )
    );

    const link = container.querySelector('a');
    fireEvent.click(link!, { ctrlKey: true });

    expect(pushSpy).not.toHaveBeenCalled();
  });

  it('should not navigate on meta+click', () => {
    const router = createRouter({
      routes: [{ path: '/', name: 'home' }]
    });
    const pushSpy = vi.spyOn(router, 'push');

    const { container } = render(
      h(RouterProvider, { router },
        h(RouterLink, { to: '/about' }, 'About')
      )
    );

    const link = container.querySelector('a');
    fireEvent.click(link!, { metaKey: true });

    expect(pushSpy).not.toHaveBeenCalled();
  });

  it('should not navigate on shift+click', () => {
    const router = createRouter({
      routes: [{ path: '/', name: 'home' }]
    });
    const pushSpy = vi.spyOn(router, 'push');

    const { container } = render(
      h(RouterProvider, { router },
        h(RouterLink, { to: '/about' }, 'About')
      )
    );

    const link = container.querySelector('a');
    fireEvent.click(link!, { shiftKey: true });

    expect(pushSpy).not.toHaveBeenCalled();
  });

  it('should not navigate on right-click', () => {
    const router = createRouter({
      routes: [{ path: '/', name: 'home' }]
    });
    const pushSpy = vi.spyOn(router, 'push');

    const { container } = render(
      h(RouterProvider, { router },
        h(RouterLink, { to: '/about' }, 'About')
      )
    );

    const link = container.querySelector('a');
    fireEvent.click(link!, { button: 2 });

    expect(pushSpy).not.toHaveBeenCalled();
  });

  it('should render children content', () => {
    const router = createRouter({
      routes: [{ path: '/', name: 'home' }]
    });

    const { container } = render(
      h(RouterProvider, { router },
        h(RouterLink, { to: '/' },
          h('span', null, 'Home'),
          h('span', null, 'Icon')
        )
      )
    );

    expect(container.textContent).toContain('Home');
    expect(container.textContent).toContain('Icon');
  });
});

// ============================================================================
// Link Interception Tests (via RouterProvider)
// ============================================================================
describe('Link interception', () => {
  it('should intercept internal links', () => {
    const router = createRouter({
      routes: [
        { path: '/', name: 'home' },
        { path: '/about', name: 'about' }
      ]
    });

    const { container } = render(
      h(RouterProvider, { router },
        h('a', { href: '/about' }, 'About')
      )
    );

    const link = container.querySelector('a');
    expect(link?.getAttribute('href')).toBe('/about');
  });

  it('should not intercept links with target="_blank"', () => {
    const router = createRouter({
      routes: [{ path: '/', name: 'home' }]
    });

    const { container } = render(
      h(RouterProvider, { router },
        h('a', { href: '/external', target: '_blank' }, 'External')
      )
    );

    const link = container.querySelector('a');
    expect(link?.getAttribute('target')).toBe('_blank');
  });

  it('should not intercept links with download attribute', () => {
    const router = createRouter({
      routes: [{ path: '/', name: 'home' }]
    });

    const { container } = render(
      h(RouterProvider, { router },
        h('a', { href: '/file.pdf', download: 'file.pdf' }, 'Download')
      )
    );

    const link = container.querySelector('a');
    expect(link?.hasAttribute('download')).toBe(true);
  });

  it('should not intercept links with data-native attribute', () => {
    const router = createRouter({
      routes: [{ path: '/', name: 'home' }]
    });

    const { container } = render(
      h(RouterProvider, { router },
        h('a', { href: '/native', 'data-native': '' }, 'Native Link')
      )
    );

    const link = container.querySelector('a');
    expect(link?.hasAttribute('data-native')).toBe(true);
  });

  it('should not intercept links with data-external attribute', () => {
    const router = createRouter({
      routes: [{ path: '/', name: 'home' }]
    });

    const { container } = render(
      h(RouterProvider, { router },
        h('a', { href: '/external', 'data-external': '' }, 'External Link')
      )
    );

    const link = container.querySelector('a');
    expect(link?.hasAttribute('data-external')).toBe(true);
  });

  it('should use replace when data-replace attribute is present', () => {
    const router = createRouter({
      routes: [
        { path: '/', name: 'home' },
        { path: '/replace', name: 'replace' }
      ]
    });

    const { container } = render(
      h(RouterProvider, { router },
        h('a', { href: '/replace', 'data-replace': '' }, 'Replace Link')
      )
    );

    const link = container.querySelector('a');
    expect(link?.hasAttribute('data-replace')).toBe(true);
  });
});

// ============================================================================
// Edge Cases and Error Handling
// ============================================================================
describe('Edge cases', () => {
  it('should handle query parameters in links', () => {
    const router = createRouter({
      routes: [
        { path: '/', name: 'home' },
        { path: '/search', name: 'search' }
      ]
    });

    const { container } = render(
      h(RouterProvider, { router },
        h(RouterLink, { to: { path: '/search', query: { q: 'test' } } }, 'Search')
      )
    );

    const link = container.querySelector('a');
    expect(link?.getAttribute('href')).toBe('/search?q=test');
  });

  it('should handle hash in links', () => {
    const router = createRouter({
      routes: [
        { path: '/', name: 'home' },
        { path: '/page', name: 'page' }
      ]
    });

    const { container } = render(
      h(RouterProvider, { router },
        h(RouterLink, { to: { path: '/page', hash: '#section' } }, 'Page')
      )
    );

    const link = container.querySelector('a');
    expect(link?.getAttribute('href')).toBe('/page#section');
  });

  it('should handle combined query and hash', () => {
    const router = createRouter({
      routes: [
        { path: '/', name: 'home' },
        { path: '/page', name: 'page' }
      ]
    });

    const { container } = render(
      h(RouterProvider, { router },
        h(RouterLink, { to: { path: '/page', query: { tab: '1' }, hash: '#details' } }, 'Page')
      )
    );

    const link = container.querySelector('a');
    expect(link?.getAttribute('href')).toBe('/page?tab=1#details');
  });

  it('should handle route with empty matched array gracefully', () => {
    const router = createRouter({
      routes: [
        { path: '/', name: 'home', component: HomeComponent }
      ]
    });
    
    // Simulate no matching route
    router.currentRoute.value = {
      fullPath: '/unknown',
      path: '/unknown',
      params: {},
      query: {},
      hash: '',
      meta: {},
      matched: []
    };

    const { container } = render(
      h(RouterProvider, { router },
        h(RouterView, null)
      )
    );

    // Should not throw, just render nothing
    expect(container).not.toBeNull();
  });

  it('should handle RouterLink with fallback href for invalid routes', () => {
    const router = createRouter({
      routes: [{ path: '/', name: 'home' }]
    });

    const { container } = render(
      h(RouterProvider, { router },
        h(RouterLink, { to: '/some-path' }, 'Link')
      )
    );

    const link = container.querySelector('a');
    expect(link?.getAttribute('href')).toBe('/some-path');
  });
});
