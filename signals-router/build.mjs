import * as esbuild from 'esbuild';

// Build ESM version
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

// Build CJS version
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

console.log('Build complete!');
