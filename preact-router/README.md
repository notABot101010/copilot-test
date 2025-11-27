# @copilot-test/preact-router

A Vue Router-inspired router for Preact with signals support. This router provides a familiar API similar to Vue Router, including navigation guards, reactive route signals, and programmatic navigation.

## Features

- 🚀 **Vue Router-like API** - Familiar API for Vue developers
- 📡 **Preact Signals** - Reactive route state using `@preact/signals`
- 🔒 **Navigation Guards** - `beforeEach`, `beforeResolve`, `afterEach` hooks
- 🎯 **Route Params & Query** - Easy access via signals (`route.params.website_id`)
- 🔄 **Programmatic Navigation** - `router.push()`, `router.replace()`, `router.back()`
- 📦 **Lazy Loading** - Support for lazy-loaded route components
- 🎨 **RouterLink** - Declarative navigation with active state classes

## Installation

```bash
npm install @copilot-test/preact-router preact @preact/signals
```

## Quick Start

### 1. Define Routes

```typescript
import { createRouter } from '@copilot-test/preact-router';

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
import { render } from 'preact';
import { RouterProvider, RouterView } from '@copilot-test/preact-router';

function App() {
  return (
    <RouterProvider router={router}>
      <nav>
        <RouterLink to="/">Home</RouterLink>
        <RouterLink to="/users">Users</RouterLink>
      </nav>
      <RouterView />
    </RouterProvider>
  );
}

render(<App />, document.getElementById('app')!);
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

Access route data using signals for automatic reactivity:

```tsx
import { useRoute, useParams, useQuery } from '@copilot-test/preact-router';

function UserProfile() {
  const route = useRoute();
  const params = useParams();
  const query = useQuery();
  
  // Access route params as signals
  // route.params.value.website_id
  // params.value.id
  
  // Access query params as signals
  // route.query.value.org_id
  // query.value.page
  
  return (
    <div>
      <h1>User: {params.value.id}</h1>
      <p>Organization: {query.value.org_id}</p>
      <p>Current path: {route.path.value}</p>
    </div>
  );
}
```

### Individual Param/Query Signals

```tsx
import { useParam, useQueryParam } from '@copilot-test/preact-router';

function Component() {
  const websiteId = useParam('website_id');
  const orgId = useQueryParam('org_id');
  
  return (
    <div>
      <p>Website: {websiteId.value}</p>
      <p>Org: {orgId.value}</p>
    </div>
  );
}
```

## Programmatic Navigation

```typescript
import { useRouter } from '@copilot-test/preact-router';

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

## RouterLink Component

```tsx
import { RouterLink } from '@copilot-test/preact-router';

function Navigation() {
  return (
    <nav>
      {/* Simple path */}
      <RouterLink to="/users">Users</RouterLink>
      
      {/* With params and query */}
      <RouterLink to={{ 
        name: 'user', 
        params: { id: '123' },
        query: { tab: 'profile' }
      }}>
        User Profile
      </RouterLink>
      
      {/* Custom active classes */}
      <RouterLink 
        to="/dashboard"
        activeClass="nav-active"
        exactActiveClass="nav-exact-active"
      >
        Dashboard
      </RouterLink>
    </nav>
  );
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
  console.log(route.meta.value); // { requiresAuth: true, roles: ['admin'] }
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
- `useRoute()` - Access reactive route object
- `useParams()` - Access route params signal
- `useParam(name)` - Access specific param signal
- `useQuery()` - Access query params signal
- `useQueryParam(name)` - Access specific query param signal
- `useMeta()` - Access route meta signal
- `usePath()` - Access path signal
- `useHash()` - Access hash signal
- `useNavigation()` - Access navigation methods

### Components

- `RouterProvider` - Provides router context to the app
- `RouterView` - Renders the matched route component
- `RouterLink` - Declarative navigation link

## TypeScript Support

This package is written in TypeScript and includes full type definitions.

```typescript
import type {
  Router,
  RouteRecord,
  RouteLocation,
  NavigationGuard,
  ReactiveRoute
} from '@copilot-test/preact-router';
```

## License

MIT
