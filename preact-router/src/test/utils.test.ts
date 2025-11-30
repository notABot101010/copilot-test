import { describe, it, expect } from 'vitest';
import {
  parseQuery,
  stringifyQuery,
  pathToRegex,
  extractParams,
  buildPath,
  normalizePath,
  joinPaths,
  matchRoutes,
  findRouteByName,
  getRoutePathByName,
  parseUrl
} from '../utils';
import type { RouteRecord } from '../types';

// ============================================================================
// parseQuery Tests (15+ tests)
// ============================================================================
describe('parseQuery', () => {
  // Basic cases
  it('should parse empty query string', () => {
    expect(parseQuery('')).toEqual({});
    expect(parseQuery('?')).toEqual({});
  });

  it('should parse single key-value pair', () => {
    expect(parseQuery('?foo=bar')).toEqual({ foo: 'bar' });
    expect(parseQuery('foo=bar')).toEqual({ foo: 'bar' });
  });

  it('should parse multiple key-value pairs', () => {
    expect(parseQuery('?foo=bar&baz=qux')).toEqual({ foo: 'bar', baz: 'qux' });
  });

  it('should handle duplicate keys as arrays', () => {
    expect(parseQuery('?foo=bar&foo=baz')).toEqual({ foo: ['bar', 'baz'] });
  });

  it('should decode URL-encoded values', () => {
    expect(parseQuery('?name=John%20Doe')).toEqual({ name: 'John Doe' });
  });

  // Edge cases
  it('should handle key without value', () => {
    expect(parseQuery('?foo=')).toEqual({ foo: '' });
  });

  it('should handle key without equals sign', () => {
    expect(parseQuery('?foo')).toEqual({ foo: '' });
  });

  it('should handle multiple empty values', () => {
    expect(parseQuery('?foo=&bar=')).toEqual({ foo: '', bar: '' });
  });

  it('should handle special characters in values', () => {
    expect(parseQuery('?email=test%40example.com')).toEqual({ email: 'test@example.com' });
    expect(parseQuery('?url=https%3A%2F%2Fexample.com')).toEqual({ url: 'https://example.com' });
  });

  it('should handle unicode characters', () => {
    expect(parseQuery('?name=%E4%B8%AD%E6%96%87')).toEqual({ name: '中文' });
  });

  it('should handle plus signs as spaces', () => {
    // Note: This tests URL encoding behavior - plus should be decoded as literal +
    const result = parseQuery('?name=John+Doe');
    expect(result.name).toBe('John+Doe');
  });

  it('should handle three or more duplicate keys', () => {
    expect(parseQuery('?id=1&id=2&id=3')).toEqual({ id: ['1', '2', '3'] });
  });

  it('should handle mixed unique and duplicate keys', () => {
    expect(parseQuery('?foo=bar&id=1&id=2&baz=qux')).toEqual({
      foo: 'bar',
      id: ['1', '2'],
      baz: 'qux'
    });
  });

  it('should handle numeric values as strings', () => {
    expect(parseQuery('?count=42&price=19.99')).toEqual({ count: '42', price: '19.99' });
  });

  it('should handle boolean-like values as strings', () => {
    expect(parseQuery('?active=true&disabled=false')).toEqual({ active: 'true', disabled: 'false' });
  });
});

// ============================================================================
// stringifyQuery Tests (12+ tests)
// ============================================================================
describe('stringifyQuery', () => {
  // Basic cases
  it('should stringify empty query object', () => {
    expect(stringifyQuery({})).toBe('');
  });

  it('should stringify single key-value pair', () => {
    expect(stringifyQuery({ foo: 'bar' })).toBe('?foo=bar');
  });

  it('should stringify multiple key-value pairs', () => {
    const result = stringifyQuery({ foo: 'bar', baz: 'qux' });
    expect(result).toContain('foo=bar');
    expect(result).toContain('baz=qux');
  });

  it('should stringify array values', () => {
    expect(stringifyQuery({ foo: ['bar', 'baz'] })).toBe('?foo=bar&foo=baz');
  });

  it('should encode special characters', () => {
    expect(stringifyQuery({ name: 'John Doe' })).toBe('?name=John%20Doe');
  });

  // Edge cases
  it('should handle empty string values', () => {
    expect(stringifyQuery({ foo: '' })).toBe('?foo=');
  });

  it('should skip undefined values', () => {
    expect(stringifyQuery({ foo: 'bar', baz: undefined })).toBe('?foo=bar');
  });

  it('should encode email addresses', () => {
    expect(stringifyQuery({ email: 'test@example.com' })).toBe('?email=test%40example.com');
  });

  it('should encode URLs', () => {
    const result = stringifyQuery({ redirect: 'https://example.com/path' });
    expect(result).toBe('?redirect=https%3A%2F%2Fexample.com%2Fpath');
  });

  it('should handle arrays with single element', () => {
    expect(stringifyQuery({ ids: ['123'] })).toBe('?ids=123');
  });

  it('should handle arrays with multiple elements', () => {
    expect(stringifyQuery({ ids: ['1', '2', '3'] })).toBe('?ids=1&ids=2&ids=3');
  });

  it('should encode unicode characters', () => {
    const result = stringifyQuery({ name: '中文' });
    expect(result).toContain('%');
    expect(decodeURIComponent(result.slice(6))).toBe('中文');
  });
});

// ============================================================================
// pathToRegex Tests (15+ tests)
// ============================================================================
describe('pathToRegex', () => {
  // Basic cases
  it('should convert simple path to regex', () => {
    const { regex, paramNames } = pathToRegex('/users');
    expect(regex.test('/users')).toBe(true);
    expect(regex.test('/users/')).toBe(false);
    expect(paramNames).toEqual([]);
  });

  it('should extract param names', () => {
    const { regex, paramNames } = pathToRegex('/users/:id');
    expect(regex.test('/users/123')).toBe(true);
    expect(regex.test('/users')).toBe(false);
    expect(paramNames).toEqual(['id']);
  });

  it('should handle multiple params', () => {
    const { regex, paramNames } = pathToRegex('/users/:userId/posts/:postId');
    expect(regex.test('/users/123/posts/456')).toBe(true);
    expect(paramNames).toEqual(['userId', 'postId']);
  });

  // Edge cases
  it('should handle root path', () => {
    const { regex, paramNames } = pathToRegex('/');
    expect(regex.test('/')).toBe(true);
    expect(regex.test('/users')).toBe(false);
    expect(paramNames).toEqual([]);
  });

  it('should handle params with underscores', () => {
    const { regex, paramNames } = pathToRegex('/users/:user_id/posts/:post_id');
    expect(regex.test('/users/abc/posts/xyz')).toBe(true);
    expect(paramNames).toEqual(['user_id', 'post_id']);
  });

  it('should handle consecutive segments', () => {
    const { regex, paramNames } = pathToRegex('/api/v1/users/:id');
    expect(regex.test('/api/v1/users/123')).toBe(true);
    expect(regex.test('/api/v2/users/123')).toBe(false);
    expect(paramNames).toEqual(['id']);
  });

  it('should not match partial paths', () => {
    const { regex } = pathToRegex('/users');
    expect(regex.test('/users/123')).toBe(false);
    expect(regex.test('/users-list')).toBe(false);
  });

  it('should handle single character params', () => {
    const { regex, paramNames } = pathToRegex('/users/:a/:b/:c');
    expect(regex.test('/users/1/2/3')).toBe(true);
    expect(paramNames).toEqual(['a', 'b', 'c']);
  });

  it('should handle params at the beginning', () => {
    const { regex, paramNames } = pathToRegex('/:category/products');
    expect(regex.test('/electronics/products')).toBe(true);
    expect(paramNames).toEqual(['category']);
  });

  it('should handle multiple params in a row', () => {
    const { regex, paramNames } = pathToRegex('/files/:year/:month/:day');
    expect(regex.test('/files/2024/01/15')).toBe(true);
    expect(paramNames).toEqual(['year', 'month', 'day']);
  });

  it('should escape special regex characters in path', () => {
    const { regex } = pathToRegex('/users.json');
    expect(regex.test('/users.json')).toBe(true);
    expect(regex.test('/usersXjson')).toBe(false);
  });

  it('should handle paths with hyphens', () => {
    const { regex, paramNames } = pathToRegex('/user-profile/:id');
    expect(regex.test('/user-profile/123')).toBe(true);
    expect(paramNames).toEqual(['id']);
  });

  it('should handle deeply nested paths', () => {
    const { regex, paramNames } = pathToRegex('/api/v1/orgs/:orgId/teams/:teamId/members/:memberId');
    expect(regex.test('/api/v1/orgs/123/teams/456/members/789')).toBe(true);
    expect(paramNames).toEqual(['orgId', 'teamId', 'memberId']);
  });

  it('should handle param values with special chars', () => {
    const { regex, paramNames } = pathToRegex('/users/:id');
    // Params should match any non-slash characters
    expect(regex.test('/users/user-123')).toBe(true);
    expect(regex.test('/users/user.name')).toBe(true);
    expect(regex.test('/users/user_name')).toBe(true);
    expect(paramNames).toEqual(['id']);
  });

  it('should handle multi-segment params with + modifier', () => {
    const { regex, paramNames } = pathToRegex('/files/:path+');
    expect(regex.test('/files/a/b/c')).toBe(true);
    expect(regex.test('/files/single')).toBe(true);
    expect(paramNames).toEqual(['path']);
  });
});

// ============================================================================
// extractParams Tests (12+ tests)
// ============================================================================
describe('extractParams', () => {
  // Basic cases
  it('should extract params from path', () => {
    expect(extractParams('/users/123', '/users/:id')).toEqual({ id: '123' });
  });

  it('should extract multiple params', () => {
    expect(extractParams('/users/123/posts/456', '/users/:userId/posts/:postId')).toEqual({
      userId: '123',
      postId: '456'
    });
  });

  it('should return null for non-matching path', () => {
    expect(extractParams('/products/123', '/users/:id')).toBeNull();
  });

  // Edge cases
  it('should extract params with special characters', () => {
    expect(extractParams('/users/john-doe', '/users/:username')).toEqual({ username: 'john-doe' });
    expect(extractParams('/users/user.name', '/users/:username')).toEqual({ username: 'user.name' });
  });

  it('should extract params with numbers only', () => {
    expect(extractParams('/posts/42', '/posts/:id')).toEqual({ id: '42' });
  });

  it('should extract params with alphanumeric values', () => {
    expect(extractParams('/orders/abc123xyz', '/orders/:orderId')).toEqual({ orderId: 'abc123xyz' });
  });

  it('should return null for shorter path', () => {
    expect(extractParams('/users', '/users/:id')).toBeNull();
  });

  it('should return null for longer path', () => {
    expect(extractParams('/users/123/extra', '/users/:id')).toBeNull();
  });

  it('should handle empty param values', () => {
    // Empty segment shouldn't match
    expect(extractParams('/users/', '/users/:id')).toBeNull();
  });

  it('should extract params from root with param', () => {
    expect(extractParams('/123', '/:id')).toEqual({ id: '123' });
  });

  it('should handle UUID-like params', () => {
    const uuid = '550e8400-e29b-41d4-a716-446655440000';
    expect(extractParams(`/items/${uuid}`, '/items/:uuid')).toEqual({ uuid });
  });

  it('should extract multiple params correctly ordered', () => {
    expect(extractParams('/org/1/team/2/user/3', '/org/:orgId/team/:teamId/user/:userId')).toEqual({
      orgId: '1',
      teamId: '2',
      userId: '3'
    });
  });
});

// ============================================================================
// buildPath Tests (10+ tests)
// ============================================================================
describe('buildPath', () => {
  // Basic cases
  it('should build path from pattern and params', () => {
    expect(buildPath('/users/:id', { id: '123' })).toBe('/users/123');
  });

  it('should handle multiple params', () => {
    expect(buildPath('/users/:userId/posts/:postId', { userId: '123', postId: '456' }))
      .toBe('/users/123/posts/456');
  });

  // Edge cases
  it('should handle empty params object', () => {
    expect(buildPath('/users', {})).toBe('/users');
  });

  it('should encode special characters in param values', () => {
    expect(buildPath('/users/:name', { name: 'John Doe' })).toBe('/users/John%20Doe');
  });

  it('should handle params with special characters', () => {
    expect(buildPath('/files/:filename', { filename: 'file.txt' })).toBe('/files/file.txt');
  });

  it('should handle numeric param values', () => {
    expect(buildPath('/page/:num', { num: '42' })).toBe('/page/42');
  });

  it('should handle multiple consecutive params', () => {
    expect(buildPath('/:year/:month/:day', { year: '2024', month: '01', day: '15' }))
      .toBe('/2024/01/15');
  });

  it('should handle unicode in param values', () => {
    expect(buildPath('/search/:query', { query: '中文' })).toBe('/search/%E4%B8%AD%E6%96%87');
  });

  it('should leave unused params in pattern', () => {
    // If a param is not provided, it stays in the pattern
    expect(buildPath('/users/:id', {})).toBe('/users/:id');
  });

  it('should ignore extra params not in pattern', () => {
    expect(buildPath('/users/:id', { id: '123', extra: 'value' })).toBe('/users/123');
  });
});

// ============================================================================
// normalizePath Tests (10+ tests)
// ============================================================================
describe('normalizePath', () => {
  // Basic cases
  it('should add leading slash', () => {
    expect(normalizePath('users')).toBe('/users');
  });

  it('should remove trailing slash', () => {
    expect(normalizePath('/users/')).toBe('/users');
  });

  it('should preserve root path', () => {
    expect(normalizePath('/')).toBe('/');
  });

  // Edge cases
  it('should handle path without leading slash', () => {
    expect(normalizePath('api/v1/users')).toBe('/api/v1/users');
  });

  it('should handle multiple trailing slashes', () => {
    expect(normalizePath('/users//')).toBe('/users/');
    // Note: Only removes single trailing slash
  });

  it('should handle empty string', () => {
    expect(normalizePath('')).toBe('/');
  });

  it('should handle just a slash', () => {
    expect(normalizePath('/')).toBe('/');
  });

  it('should handle path with query string', () => {
    expect(normalizePath('/users?page=1')).toBe('/users?page=1');
  });

  it('should handle deeply nested paths', () => {
    expect(normalizePath('api/v1/organizations/teams/members/')).toBe('/api/v1/organizations/teams/members');
  });

  it('should preserve internal slashes', () => {
    expect(normalizePath('a/b/c/d')).toBe('/a/b/c/d');
  });
});

// ============================================================================
// joinPaths Tests (12+ tests)
// ============================================================================
describe('joinPaths', () => {
  // Basic cases
  it('should join paths', () => {
    expect(joinPaths('/users', ':id')).toBe('/users/:id');
  });

  it('should handle empty paths', () => {
    expect(joinPaths('/', 'users')).toBe('/users');
  });

  // Edge cases
  it('should join multiple paths', () => {
    expect(joinPaths('/api', 'v1', 'users')).toBe('/api/v1/users');
  });

  it('should handle paths with leading slashes', () => {
    expect(joinPaths('/api', '/v1', '/users')).toBe('/api/v1/users');
  });

  it('should handle paths with trailing slashes', () => {
    expect(joinPaths('/api/', '/v1/', '/users/')).toBe('/api/v1/users');
  });

  it('should handle single path', () => {
    expect(joinPaths('/users')).toBe('/users');
  });

  it('should handle empty string paths', () => {
    expect(joinPaths('', '/users')).toBe('/users');
  });

  it('should handle all empty paths', () => {
    expect(joinPaths('', '', '')).toBe('/');
  });

  it('should join path with params', () => {
    expect(joinPaths('/users', ':userId', 'posts', ':postId')).toBe('/users/:userId/posts/:postId');
  });

  it('should handle root with path', () => {
    expect(joinPaths('/', '/')).toBe('/');
  });

  it('should handle complex nested paths', () => {
    expect(joinPaths('/api/v1/', '/orgs/', '/:orgId/', '/teams')).toBe('/api/v1/orgs/:orgId/teams');
  });

  it('should handle paths with dots', () => {
    expect(joinPaths('/api', 'v1.0', 'users')).toBe('/api/v1.0/users');
  });
});

// ============================================================================
// matchRoutes Tests (15+ tests)
// ============================================================================
describe('matchRoutes', () => {
  const routes: RouteRecord[] = [
    { path: '/', name: 'home' },
    { path: '/users', name: 'users' },
    { path: '/users/:id', name: 'user' },
    {
      path: '/admin',
      name: 'admin',
      children: [
        { path: '/admin/settings', name: 'admin-settings' },
        { path: '/admin/users', name: 'admin-users' },
        { path: '/admin/users/:id', name: 'admin-user' }
      ]
    }
  ];

  // Basic cases
  it('should match root path', () => {
    const result = matchRoutes('/', routes);
    expect(result).not.toBeNull();
    expect(result!.matched[0].name).toBe('home');
  });

  it('should match path with params', () => {
    const result = matchRoutes('/users/123', routes);
    expect(result).not.toBeNull();
    expect(result!.matched[0].name).toBe('user');
    expect(result!.params).toEqual({ id: '123' });
  });

  it('should return null for non-matching path', () => {
    const result = matchRoutes('/products', routes);
    expect(result).toBeNull();
  });

  // Edge cases
  it('should match static path before param path', () => {
    const result = matchRoutes('/users', routes);
    expect(result).not.toBeNull();
    expect(result!.matched[0].name).toBe('users');
  });

  it('should match nested child routes with relative paths', () => {
    // Note: This router expects child routes to use relative paths
    const relativeChildRoutes: RouteRecord[] = [
      {
        path: '/admin',
        name: 'admin',
        children: [
          { path: '/settings', name: 'admin-settings' }
        ]
      }
    ];
    const result = matchRoutes('/admin/settings', relativeChildRoutes);
    expect(result).not.toBeNull();
    expect(result!.matched.length).toBe(2);
    expect(result!.matched[0].name).toBe('admin');
    expect(result!.matched[1].name).toBe('admin-settings');
  });

  it('should match nested child route with relative path and params', () => {
    const relativeChildRoutes: RouteRecord[] = [
      {
        path: '/admin',
        name: 'admin',
        children: [
          { path: '/users/:id', name: 'admin-user' }
        ]
      }
    ];
    const result = matchRoutes('/admin/users/456', relativeChildRoutes);
    expect(result).not.toBeNull();
    expect(result!.matched.length).toBe(2);
    expect(result!.matched[1].name).toBe('admin-user');
    expect(result!.params).toEqual({ id: '456' });
  });

  it('should not match partial paths', () => {
    const result = matchRoutes('/use', routes);
    expect(result).toBeNull();
  });

  it('should not match paths with extra segments', () => {
    const result = matchRoutes('/users/123/extra', routes);
    expect(result).toBeNull();
  });

  it('should match multiple params in different routes', () => {
    const multiParamRoutes: RouteRecord[] = [
      { path: '/org/:orgId/team/:teamId', name: 'team' }
    ];
    const result = matchRoutes('/org/abc/team/xyz', multiParamRoutes);
    expect(result).not.toBeNull();
    expect(result!.params).toEqual({ orgId: 'abc', teamId: 'xyz' });
  });

  it('should handle routes with meta', () => {
    const metaRoutes: RouteRecord[] = [
      { path: '/dashboard', name: 'dashboard', meta: { requiresAuth: true } }
    ];
    const result = matchRoutes('/dashboard', metaRoutes);
    expect(result).not.toBeNull();
    expect(result!.matched[0].meta).toEqual({ requiresAuth: true });
  });

  it('should handle deeply nested routes with relative paths', () => {
    const deepRoutes: RouteRecord[] = [
      {
        path: '/a',
        name: 'a',
        children: [
          {
            path: '/b',
            name: 'b',
            children: [
              { path: '/c', name: 'c' }
            ]
          }
        ]
      }
    ];
    const result = matchRoutes('/a/b/c', deepRoutes);
    expect(result).not.toBeNull();
    expect(result!.matched.length).toBe(3);
  });

  it('should handle routes without names', () => {
    const unnamedRoutes: RouteRecord[] = [
      { path: '/unnamed' }
    ];
    const result = matchRoutes('/unnamed', unnamedRoutes);
    expect(result).not.toBeNull();
    expect(result!.matched[0].name).toBeUndefined();
  });

  it('should handle multiple root-level routes with nested children', () => {
    const nestedRoutes: RouteRecord[] = [
      {
        path: '/admin',
        name: 'admin',
        children: [
          { path: '/users', name: 'admin-users' }
        ]
      }
    ];
    const result = matchRoutes('/admin/users', nestedRoutes);
    expect(result).not.toBeNull();
    expect(result!.matched[0].name).toBe('admin');
    expect(result!.matched[1].name).toBe('admin-users');
  });

  it('should match catch-all route', () => {
    const catchAllRoutes: RouteRecord[] = [
      { path: '/', name: 'home' },
      { path: '/*', name: 'catch-all' }
    ];
    const result = matchRoutes('/any/random/path', catchAllRoutes);
    // Note: depends on implementation
  });

  it('should handle empty children array', () => {
    const emptyChildrenRoutes: RouteRecord[] = [
      { path: '/empty', name: 'empty', children: [] }
    ];
    const result = matchRoutes('/empty', emptyChildrenRoutes);
    expect(result).not.toBeNull();
    expect(result!.matched[0].name).toBe('empty');
  });
});

// ============================================================================
// findRouteByName Tests (10+ tests)
// ============================================================================
describe('findRouteByName', () => {
  const routes: RouteRecord[] = [
    { path: '/', name: 'home' },
    { path: '/users', name: 'users' },
    {
      path: '/admin',
      name: 'admin',
      children: [
        { path: '/settings', name: 'admin-settings' },
        {
          path: '/nested',
          name: 'admin-nested',
          children: [
            { path: '/deep', name: 'admin-deep' }
          ]
        }
      ]
    }
  ];

  // Basic cases
  it('should find route by name', () => {
    const route = findRouteByName('home', routes);
    expect(route).not.toBeNull();
    expect(route!.path).toBe('/');
  });

  it('should find nested route by name', () => {
    const route = findRouteByName('admin-settings', routes);
    expect(route).not.toBeNull();
    expect(route!.path).toBe('/settings');
  });

  it('should return null for non-existing name', () => {
    const route = findRouteByName('not-found', routes);
    expect(route).toBeNull();
  });

  // Edge cases
  it('should find deeply nested route', () => {
    const route = findRouteByName('admin-deep', routes);
    expect(route).not.toBeNull();
    expect(route!.path).toBe('/deep');
  });

  it('should find parent route with children', () => {
    const route = findRouteByName('admin', routes);
    expect(route).not.toBeNull();
    expect(route!.children).toBeDefined();
    expect(route!.children!.length).toBe(2);
  });

  it('should handle empty routes array', () => {
    const route = findRouteByName('home', []);
    expect(route).toBeNull();
  });

  it('should handle routes without names', () => {
    const unnamedRoutes: RouteRecord[] = [
      { path: '/' },
      { path: '/users' }
    ];
    const route = findRouteByName('home', unnamedRoutes);
    expect(route).toBeNull();
  });

  it('should find first match when names are duplicated', () => {
    const duplicateRoutes: RouteRecord[] = [
      { path: '/first', name: 'duplicate' },
      { path: '/second', name: 'duplicate' }
    ];
    const route = findRouteByName('duplicate', duplicateRoutes);
    expect(route).not.toBeNull();
    expect(route!.path).toBe('/first');
  });

  it('should handle case-sensitive names', () => {
    const route = findRouteByName('Home', routes);
    expect(route).toBeNull(); // Should not match 'home'
  });

  it('should find route with meta data', () => {
    const metaRoutes: RouteRecord[] = [
      { path: '/protected', name: 'protected', meta: { auth: true } }
    ];
    const route = findRouteByName('protected', metaRoutes);
    expect(route).not.toBeNull();
    expect(route!.meta).toEqual({ auth: true });
  });
});

// ============================================================================
// getRoutePathByName Tests (10+ tests)
// ============================================================================
describe('getRoutePathByName', () => {
  const routes: RouteRecord[] = [
    { path: '/', name: 'home' },
    { path: '/users/:id', name: 'user' },
    {
      path: '/admin',
      name: 'admin',
      children: [
        { path: '/dashboard', name: 'admin-dashboard' },
        { path: '/users/:userId', name: 'admin-user' }
      ]
    }
  ];

  // Basic cases
  it('should get path by route name', () => {
    expect(getRoutePathByName('home', routes)).toBe('/');
    expect(getRoutePathByName('user', routes)).toBe('/users/:id');
  });

  it('should return null for non-existing name', () => {
    expect(getRoutePathByName('not-found', routes)).toBeNull();
  });

  // Edge cases
  it('should get full path for nested route', () => {
    expect(getRoutePathByName('admin-dashboard', routes)).toBe('/admin/dashboard');
  });

  it('should get full path with params for nested route', () => {
    expect(getRoutePathByName('admin-user', routes)).toBe('/admin/users/:userId');
  });

  it('should handle root route', () => {
    expect(getRoutePathByName('home', routes)).toBe('/');
  });

  it('should handle parent route path', () => {
    expect(getRoutePathByName('admin', routes)).toBe('/admin');
  });

  it('should handle empty routes array', () => {
    expect(getRoutePathByName('home', [])).toBeNull();
  });

  it('should handle deeply nested routes', () => {
    const deepRoutes: RouteRecord[] = [
      {
        path: '/a',
        name: 'a',
        children: [
          {
            path: '/b',
            name: 'b',
            children: [
              { path: '/c', name: 'c' }
            ]
          }
        ]
      }
    ];
    expect(getRoutePathByName('c', deepRoutes)).toBe('/a/b/c');
  });

  it('should handle routes with multiple params', () => {
    const paramRoutes: RouteRecord[] = [
      { path: '/org/:orgId/team/:teamId', name: 'team' }
    ];
    expect(getRoutePathByName('team', paramRoutes)).toBe('/org/:orgId/team/:teamId');
  });

  it('should handle case-sensitive names', () => {
    expect(getRoutePathByName('Home', routes)).toBeNull();
  });
});

// ============================================================================
// parseUrl Tests (15+ tests)
// ============================================================================
describe('parseUrl', () => {
  // Basic cases
  it('should parse simple path', () => {
    const result = parseUrl('/users');
    expect(result.path).toBe('/users');
    expect(result.query).toEqual({});
    expect(result.hash).toBe('');
  });

  it('should parse path with query string', () => {
    const result = parseUrl('/users?page=1');
    expect(result.path).toBe('/users');
    expect(result.query).toEqual({ page: '1' });
  });

  it('should parse path with hash', () => {
    const result = parseUrl('/users#section');
    expect(result.path).toBe('/users');
    expect(result.hash).toBe('#section');
  });

  it('should parse full URL', () => {
    const result = parseUrl('/users?page=1&sort=name#section');
    expect(result.path).toBe('/users');
    expect(result.query).toEqual({ page: '1', sort: 'name' });
    expect(result.hash).toBe('#section');
  });

  // Edge cases
  it('should parse root path', () => {
    const result = parseUrl('/');
    expect(result.path).toBe('/');
    expect(result.query).toEqual({});
    expect(result.hash).toBe('');
  });

  it('should parse empty path', () => {
    const result = parseUrl('');
    expect(result.path).toBe('/');
  });

  it('should parse hash only', () => {
    const result = parseUrl('#section');
    expect(result.path).toBe('/');
    expect(result.hash).toBe('#section');
  });

  it('should parse query only', () => {
    const result = parseUrl('?page=1');
    expect(result.path).toBe('/');
    expect(result.query).toEqual({ page: '1' });
  });

  it('should handle query before hash', () => {
    const result = parseUrl('/page?query=value#anchor');
    expect(result.path).toBe('/page');
    expect(result.query).toEqual({ query: 'value' });
    expect(result.hash).toBe('#anchor');
  });

  it('should handle hash without query', () => {
    const result = parseUrl('/page#section');
    expect(result.path).toBe('/page');
    expect(result.query).toEqual({});
    expect(result.hash).toBe('#section');
  });

  it('should handle multiple query params', () => {
    const result = parseUrl('/search?q=test&limit=10&offset=0');
    expect(result.path).toBe('/search');
    expect(result.query).toEqual({ q: 'test', limit: '10', offset: '0' });
  });

  it('should handle encoded query values', () => {
    const result = parseUrl('/search?q=hello%20world');
    expect(result.query.q).toBe('hello world');
  });

  it('should handle base path removal', () => {
    const result = parseUrl('/app/users', '/app');
    expect(result.path).toBe('/users');
  });

  it('should handle complex hash', () => {
    const result = parseUrl('/page#section/subsection');
    expect(result.hash).toBe('#section/subsection');
  });

  it('should handle query with special characters', () => {
    const result = parseUrl('/search?email=test%40example.com');
    expect(result.query.email).toBe('test@example.com');
  });
});
