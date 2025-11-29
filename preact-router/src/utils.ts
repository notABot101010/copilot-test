import type { RouteParams, RouteQuery, RouteRecord, RouteLocation } from './types';

/**
 * Parse query string into an object
 */
export function parseQuery(queryString: string): RouteQuery {
  const query: RouteQuery = {};
  
  if (!queryString || queryString === '?') {
    return query;
  }
  
  const search = queryString.startsWith('?') ? queryString.slice(1) : queryString;
  
  for (const pair of search.split('&')) {
    const [key, value] = pair.split('=').map(decodeURIComponent);
    if (key) {
      const existing = query[key];
      if (existing !== undefined) {
        if (Array.isArray(existing)) {
          existing.push(value ?? '');
        } else {
          query[key] = [existing, value ?? ''];
        }
      } else {
        query[key] = value ?? '';
      }
    }
  }
  
  return query;
}

/**
 * Stringify query object into a query string
 */
export function stringifyQuery(query: RouteQuery): string {
  const pairs: string[] = [];
  
  for (const [key, value] of Object.entries(query)) {
    if (value === undefined) continue;
    
    if (Array.isArray(value)) {
      for (const v of value) {
        pairs.push(`${encodeURIComponent(key)}=${encodeURIComponent(v)}`);
      }
    } else {
      pairs.push(`${encodeURIComponent(key)}=${encodeURIComponent(value)}`);
    }
  }
  
  return pairs.length > 0 ? `?${pairs.join('&')}` : '';
}

/**
 * Convert a route path pattern to a regex and extract param names
 */
export function pathToRegex(path: string): { regex: RegExp; paramNames: string[] } {
  const paramNames: string[] = [];
  const paramPositions: { name: string; index: number; isMulti: boolean }[] = [];
  
  // Find all parameter positions in order of their appearance in the path
  let match: RegExpExecArray | null;
  const multiParamRegex = /:([a-zA-Z_][a-zA-Z0-9_]*)\+/g;
  const singleParamRegex = /:([a-zA-Z_][a-zA-Z0-9_]*)(?!\+)/g;
  
  // First pass: find multi-segment params (:param+)
  while ((match = multiParamRegex.exec(path)) !== null) {
    paramPositions.push({ name: match[1], index: match.index, isMulti: true });
  }
  
  // Second pass: find single-segment params (:param)
  while ((match = singleParamRegex.exec(path)) !== null) {
    // Skip if this is part of a multi-segment param (we already captured it)
    const matchIndex = match.index;
    const isMulti = paramPositions.some(p => p.index === matchIndex);
    if (!isMulti) {
      paramPositions.push({ name: match[1], index: matchIndex, isMulti: false });
    }
  }
  
  // Sort by position in the path string to maintain correct order
  paramPositions.sort((a, b) => a.index - b.index);
  
  // Extract param names in order
  for (const p of paramPositions) {
    paramNames.push(p.name);
  }
  
  // Build the regex string
  let regexStr = path
    // Escape dots and other special chars (but not + which we handle separately)
    .replace(/[.?^${}()|[\]\\]/g, '\\$&')
    // Replace :paramName+ with capture groups that match multiple path segments
    .replace(/:([a-zA-Z_][a-zA-Z0-9_]*)\+/g, '(.+)')
    // Replace :paramName with capture groups that match a single segment
    .replace(/:([a-zA-Z_][a-zA-Z0-9_]*)/g, '([^/]+)')
    // Handle wildcard/catch-all routes
    .replace(/\*/g, '.*');
  
  // Ensure the regex matches the full path
  regexStr = `^${regexStr}$`;
  
  return { regex: new RegExp(regexStr), paramNames };
}

/**
 * Extract params from a path using a route pattern
 */
export function extractParams(path: string, pattern: string): RouteParams | null {
  const { regex, paramNames } = pathToRegex(pattern);
  const match = path.match(regex);
  
  if (!match) {
    return null;
  }
  
  const params: RouteParams = {};
  paramNames.forEach((name, index) => {
    params[name] = match[index + 1] || '';
  });
  
  return params;
}

/**
 * Build a path from a pattern and params
 */
export function buildPath(pattern: string, params: RouteParams): string {
  let path = pattern;
  
  for (const [key, value] of Object.entries(params)) {
    path = path.replace(`:${key}`, encodeURIComponent(value));
  }
  
  return path;
}

/**
 * Normalize a path (ensure it starts with /, remove trailing slash except for root)
 */
export function normalizePath(path: string): string {
  // Ensure path starts with /
  if (!path.startsWith('/')) {
    path = '/' + path;
  }
  
  // Remove trailing slash except for root
  if (path !== '/' && path.endsWith('/')) {
    path = path.slice(0, -1);
  }
  
  return path;
}

/**
 * Join base path with route path
 */
export function joinPaths(...paths: string[]): string {
  return normalizePath(
    paths
      .map((p, i) => {
        if (i === 0) return p.replace(/\/$/, '');
        return p.replace(/^\//, '').replace(/\/$/, '');
      })
      .filter(p => p)
      .join('/')
  );
}

/**
 * Match a path against route records and return matched routes
 */
export function matchRoutes(
  path: string,
  routes: RouteRecord[],
  parentPath: string = ''
): { matched: RouteRecord[]; params: RouteParams } | null {
  for (const route of routes) {
    const fullPattern = joinPaths(parentPath, route.path);
    
    // Try to match this route
    const params = extractParams(path, fullPattern);
    
    if (params !== null) {
      // Check for child routes
      if (route.children && route.children.length > 0) {
        const childMatch = matchRoutes(path, route.children, fullPattern);
        if (childMatch) {
          return {
            matched: [route, ...childMatch.matched],
            params: { ...params, ...childMatch.params }
          };
        }
      }
      
      return { matched: [route], params };
    }
    
    // If this route has children, try matching them even if parent doesn't match exactly
    // This supports nested routes where parent is just a prefix
    if (route.children && route.children.length > 0) {
      const childMatch = matchRoutes(path, route.children, fullPattern.replace(/\/?$/, ''));
      if (childMatch) {
        return {
          matched: [route, ...childMatch.matched],
          params: childMatch.params
        };
      }
    }
  }
  
  return null;
}

/**
 * Find a route by name
 */
export function findRouteByName(name: string, routes: RouteRecord[]): RouteRecord | null {
  for (const route of routes) {
    if (route.name === name) {
      return route;
    }
    
    if (route.children) {
      const found = findRouteByName(name, route.children);
      if (found) {
        return found;
      }
    }
  }
  
  return null;
}

/**
 * Get the full path pattern for a named route
 */
export function getRoutePathByName(
  name: string,
  routes: RouteRecord[],
  parentPath: string = ''
): string | null {
  for (const route of routes) {
    const fullPath = joinPaths(parentPath, route.path);
    
    if (route.name === name) {
      return fullPath;
    }
    
    if (route.children) {
      const found = getRoutePathByName(name, route.children, fullPath);
      if (found) {
        return found;
      }
    }
  }
  
  return null;
}

/**
 * Parse a URL into its components
 */
export function parseUrl(url: string, base: string = ''): { path: string; query: RouteQuery; hash: string } {
  let path = url;
  let queryString = '';
  let hash = '';
  
  // Extract hash
  const hashIndex = path.indexOf('#');
  if (hashIndex !== -1) {
    hash = path.slice(hashIndex);
    path = path.slice(0, hashIndex);
  }
  
  // Extract query string
  const queryIndex = path.indexOf('?');
  if (queryIndex !== -1) {
    queryString = path.slice(queryIndex);
    path = path.slice(0, queryIndex);
  }
  
  // Remove base from path
  if (base && path.startsWith(base)) {
    path = path.slice(base.length) || '/';
  }
  
  path = normalizePath(path);
  
  return {
    path,
    query: parseQuery(queryString),
    hash
  };
}

/**
 * Create a full path from components
 */
export function createFullPath(path: string, query: RouteQuery, hash: string): string {
  return `${path}${stringifyQuery(query)}${hash}`;
}

/**
 * Create an empty route location
 */
export function createEmptyRouteLocation(): RouteLocation {
  return {
    fullPath: '/',
    path: '/',
    params: {},
    query: {},
    hash: '',
    name: undefined,
    meta: {},
    matched: []
  };
}
