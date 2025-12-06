# @copilot-test/signals-router

A Vue Router-inspired router for React with signals support. This router provides a familiar API similar to Vue Router, including navigation guards, reactive route signals, and programmatic navigation.

## Features

- 🚀 **Vue Router-like API** - Familiar API for Vue developers
- 📡 **React Signals** - Reactive route state using `@preact/signals-react`
- 🔒 **Navigation Guards** - `beforeEach`, `beforeResolve`, `afterEach` hooks
- 🎯 **Route Params & Query** - Easy access via signals (`route.value.params.website_id`)
- 🔄 **Programmatic Navigation** - `router.push()`, `router.replace()`, `router.back()`
- 📦 **Lazy Loading** - Support for lazy-loaded route components
- 🔗 **Native Links** - Works with regular `<a>` tags, no special component needed

## Installation

```bash
npm install @copilot-test/signals-router react @preact/signals-react
```

### Vite Configuration

When using this library with Vite, import and use the `signalsRouterPlugin` in your `vite.config.ts`:

```typescript
import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import { signalsRouterPlugin } from '@copilot-test/signals-router'

export default defineConfig({
  plugins: [
    react(),
    signalsRouterPlugin(), // Automatically configures dedupe for React and signals
  ],
})
```

The plugin automatically configures Vite to dedupe `react`, `react-dom`, and `@preact/signals-react`, preventing multiple instances of these libraries which can cause issues with React hooks and signal subscriptions.

## Quick Start

### 1. Define Routes

```typescript
import { createRouter } from '@copilot-test/signals-router';

const router = createRouter({
  routes: [
    { path: '/', name: 'home', component: Home },
    { path: '/users', name: 'users', component: Users },
    { path: '/users/:id', name: 'user', component: UserDetail },
    { 
      path: '/dashboard', 
      name: 'dashboard', 
      component: Dashboard,
      meta: { requiresAuth: true }
    }
  ],
  mode: 'history' // or 'hash'
});
```

### 2. Add Router Provider

```tsx
import { createRoot } from 'react-dom/client';
import { RouterProvider, RouterView } from '@copilot-test/signals-router';

function App() {
  return (
    <RouterProvider router={router}>
      <nav>
        {/* Use regular <a> tags - the router intercepts clicks automatically */}
        <a href="/">Home</a>
        <a href="/users">Users</a>
        <a href="/dashboard">Dashboard</a>
      </nav>
      <RouterView />
    </RouterProvider>
  );
}

const root = createRoot(document.getElementById('app')!);
root.render(<App />);
```

## Using Regular Links

The router automatically intercepts clicks on `<a>` tags within the `RouterProvider`. No special component is needed:

```tsx
function Navigation() {
  return (
    <nav>
      {/* All these work automatically */}
      <a href="/users">Users</a>
      <a href="/users/123">User Detail</a>
      <a href="/search?q=hello">Search</a>
      
      {/* Use data-replace for replace navigation (no history entry) */}
      <a href="/login" data-replace>Login</a>
      
      {/* External links work normally */}
      <a href="https://google.com">Google</a>
      <a href="/api/download" target="_blank">Download</a>
      
      {/* Opt-out of SPA navigation with data-native */}
      <a href="/legacy-page" data-native>Legacy Page</a>
    </nav>
  );
}
```

## Navigation Guards

### Global Before Guard (Authentication)

```typescript
// Similar to Vue Router's router.beforeEach
router.beforeEach(async (to, from) => {
  // Check if route requires authentication
  if (to.meta.requiresAuth) {
    const isAuthenticated = await checkAuth();
    
    if (!isAuthenticated) {
      // Redirect to login
      return '/login';
    }
  }
  
  // Continue navigation
  return true;
});
```

### Route-Specific Guards

```typescript
const routes = [
  {
    path: '/admin',
    component: AdminPanel,
    beforeEnter: (to, from) => {
      if (!isAdmin()) {
        return '/unauthorized';
      }
    }
  }
];
```

## Reactive Route Signals

Access all route data using the `useRoute()` hook which returns a signal:

```tsx
import { useRoute } from '@copilot-test/signals-router';

function UserProfile() {
  const route = useRoute();
  
  // Access route params
  // route.value.params.website_id
  // route.value.params.id
  
  // Access query params
  // route.value.query.org_id
  // route.value.query.page
  
  // Access other route properties
  // route.value.path      - current path
  // route.value.hash      - URL hash
  // route.value.meta      - route meta data
  // route.value.fullPath  - full path with query and hash
  // route.value.name      - route name
  
  return (
    <div>
      <h1>User: {route.value.params.id}</h1>
      <p>Organization: {route.value.query.org_id}</p>
      <p>Current path: {route.value.path}</p>
    </div>
  );
}
```

## Programmatic Navigation

```typescript
import { useRouter } from '@copilot-test/signals-router';

function MyComponent() {
  const router = useRouter();
  
  const handleClick = async () => {
    // Navigate by path
    await router.push('/users/123');
    
    // Navigate by name with params
    await router.push({ 
      name: 'user', 
      params: { id: '123' },
      query: { org_id: 'abc' }
    });
    
    // Replace current route (no history entry)
    await router.replace('/login');
    
    // Go back/forward
    router.back();
    router.forward();
    router.go(-2);
  };
  
  return <button onClick={handleClick}>Navigate</button>;
}
```

## Lazy Loading

```typescript
const routes = [
  {
    path: '/settings',
    name: 'settings',
    lazyComponent: () => import('./pages/Settings')
  }
];
```

## Route Meta

```typescript
const routes = [
  {
    path: '/admin',
    component: Admin,
    meta: {
      requiresAuth: true,
      roles: ['admin']
    }
  }
];

// Access in guards
router.beforeEach((to) => {
  if (to.meta.requiresAuth) {
    // Check authentication
  }
});

// Access in components
function Component() {
  const route = useRoute();
  console.log(route.value.meta); // { requiresAuth: true, roles: ['admin'] }
}
```

## API Reference

### `createRouter(options)`

Creates a new router instance.

**Options:**
- `routes: RouteRecord[]` - Array of route definitions
- `mode?: 'history' | 'hash'` - Router mode (default: 'history')
- `base?: string` - Base path for all routes
- `scrollBehavior?: Function` - Custom scroll behavior

### `Router` Instance

- `currentRoute: Signal<RouteLocation>` - Current route as a signal
- `push(to): Promise<void>` - Navigate to a new route
- `replace(to): Promise<void>` - Replace current route
- `back()` - Go back in history
- `forward()` - Go forward in history
- `go(delta)` - Go to a specific point in history
- `beforeEach(guard): () => void` - Register global before guard
- `beforeResolve(guard): () => void` - Register before resolve guard
- `afterEach(hook): () => void` - Register after navigation hook
- `onError(handler): () => void` - Register error handler
- `hasRoute(name): boolean` - Check if route exists
- `addRoute(parent, route): () => void` - Add a route dynamically
- `removeRoute(name): void` - Remove a route

### Hooks

- `useRouter()` - Access router instance
- `useRoute()` - Access reactive route signal. Access properties via `route.value`:
  - `route.value.params` - Route params
  - `route.value.query` - Query params
  - `route.value.path` - Path
  - `route.value.hash` - Hash
  - `route.value.meta` - Meta
  - `route.value.fullPath` - Full path
  - `route.value.name` - Route name
- `useNavigation()` - Access navigation methods (push, replace, back, forward, go)

### Components

- `RouterProvider` - Provides router context and intercepts link clicks
- `RouterView` - Renders the matched route component
  - `notFound` prop - Component to render when no route matches (404 page)
- `RouterLink` - Optional declarative navigation link with active state classes

### RouterView NotFound Support

You can specify a component to render when no route matches:

```tsx
import { RouterView } from '@copilot-test/preact-router';
import { NotFoundPage } from './pages/NotFoundPage';

function App() {
  return (
    <RouterProvider router={router}>
      <RouterView notFound={NotFoundPage} />
    </RouterProvider>
  );
}
```

### Link Attributes

When using regular `<a>` tags:

- `data-replace` - Use replace navigation instead of push
- `data-native` or `data-external` - Skip SPA navigation, use native browser navigation
- `target="_blank"` - Opens in new tab (native behavior)
- `download` - Download attribute (native behavior)

## TypeScript Support

This package is written in TypeScript and includes full type definitions.

```typescript
import type {
  Router,
  RouteRecord,
  RouteLocation,
  NavigationGuard,
  ReactiveRoute
} from '@copilot-test/signals-router';
```

## License

MIT
