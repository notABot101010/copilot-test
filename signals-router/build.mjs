import * as esbuild from 'esbuild';

// Build main library - ESM version
await esbuild.build({
  entryPoints: ['src/index.ts'],
  bundle: true,
  format: 'esm',
  outfile: 'dist/index.js',
  external: ['react', 'react-dom', 'react/*', '@preact/signals-react', '@preact/signals-core'],
  platform: 'browser',
  target: 'es2020',
  sourcemap: true,
});

// Build main library - CJS version
await esbuild.build({
  entryPoints: ['src/index.ts'],
  bundle: true,
  format: 'cjs',
  outfile: 'dist/index.cjs',
  external: ['react', 'react-dom', 'react/*', '@preact/signals-react', '@preact/signals-core'],
  platform: 'browser',
  target: 'es2020',
  sourcemap: true,
});

// Build Vite plugin - ESM version
await esbuild.build({
  entryPoints: ['src/vite.ts'],
  bundle: true,
  format: 'esm',
  outfile: 'dist/vite.js',
  external: ['vite'],
  platform: 'node',
  target: 'node14',
  sourcemap: true,
});

// Build Vite plugin - CJS version
await esbuild.build({
  entryPoints: ['src/vite.ts'],
  bundle: true,
  format: 'cjs',
  outfile: 'dist/vite.cjs',
  external: ['vite'],
  platform: 'node',
  target: 'node14',
  sourcemap: true,
});

console.log('Build complete!');
