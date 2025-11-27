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

describe('parseQuery', () => {
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
});

describe('stringifyQuery', () => {
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
});

describe('pathToRegex', () => {
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
});

describe('extractParams', () => {
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
});

describe('buildPath', () => {
  it('should build path from pattern and params', () => {
    expect(buildPath('/users/:id', { id: '123' })).toBe('/users/123');
  });

  it('should handle multiple params', () => {
    expect(buildPath('/users/:userId/posts/:postId', { userId: '123', postId: '456' }))
      .toBe('/users/123/posts/456');
  });
});

describe('normalizePath', () => {
  it('should add leading slash', () => {
    expect(normalizePath('users')).toBe('/users');
  });

  it('should remove trailing slash', () => {
    expect(normalizePath('/users/')).toBe('/users');
  });

  it('should preserve root path', () => {
    expect(normalizePath('/')).toBe('/');
  });
});

describe('joinPaths', () => {
  it('should join paths', () => {
    expect(joinPaths('/users', ':id')).toBe('/users/:id');
  });

  it('should handle empty paths', () => {
    expect(joinPaths('/', 'users')).toBe('/users');
  });
});

describe('matchRoutes', () => {
  const routes: RouteRecord[] = [
    { path: '/', name: 'home' },
    { path: '/users', name: 'users' },
    { path: '/users/:id', name: 'user' },
    {
      path: '/admin',
      name: 'admin',
      children: [
        { path: '/admin/settings', name: 'admin-settings' }
      ]
    }
  ];

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
});

describe('findRouteByName', () => {
  const routes: RouteRecord[] = [
    { path: '/', name: 'home' },
    { path: '/users', name: 'users' },
    {
      path: '/admin',
      name: 'admin',
      children: [
        { path: '/settings', name: 'admin-settings' }
      ]
    }
  ];

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
});

describe('getRoutePathByName', () => {
  const routes: RouteRecord[] = [
    { path: '/', name: 'home' },
    { path: '/users/:id', name: 'user' }
  ];

  it('should get path by route name', () => {
    expect(getRoutePathByName('home', routes)).toBe('/');
    expect(getRoutePathByName('user', routes)).toBe('/users/:id');
  });

  it('should return null for non-existing name', () => {
    expect(getRoutePathByName('not-found', routes)).toBeNull();
  });
});

describe('parseUrl', () => {
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
});
