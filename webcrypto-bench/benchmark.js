// WebCrypto vs ChaCha WASM Benchmark
// Benchmarks AES-256-GCM (WebCrypto) against ChaCha8/12/20 (WASM)

const DATA_SIZES = [64, 256, 1024, 4096, 16384, 65536];
const ITERATIONS = 1000;

// Detect environment
const isBrowser = typeof window !== 'undefined';
const isNode = typeof process !== 'undefined' && process.versions && process.versions.node;

let crypto;
let chachaModule;

async function initCrypto() {
  if (isBrowser) {
    crypto = window.crypto;
  } else if (isNode) {
    crypto = (await import('node:crypto')).webcrypto;
  }
}

async function initChacha() {
  if (isBrowser) {
    const init = (await import('./pkg/chacha_browser.js')).default;
    chachaModule = await init();
  } else if (isNode) {
    const { readFile } = await import('node:fs/promises');
    const { fileURLToPath } = await import('node:url');
    const { dirname, join } = await import('node:path');
    
    const currentDir = dirname(fileURLToPath(import.meta.url));
    const wasmPath = join(currentDir, 'pkg', 'chacha_browser_bg.wasm');
    const wasmBuffer = await readFile(wasmPath);
    
    const wasmModule = await import('./pkg/chacha_browser.js');
    await wasmModule.default({ module_or_path: wasmBuffer });
    chachaModule = wasmModule;
  }
}

function generateRandomBytes(length) {
  const buffer = new Uint8Array(length);
  crypto.getRandomValues(buffer);
  return buffer;
}

async function benchmarkAES256GCM(dataSize, iterations) {
  const data = generateRandomBytes(dataSize);
  const key = await crypto.subtle.generateKey(
    { name: 'AES-GCM', length: 256 },
    true,
    ['encrypt', 'decrypt']
  );
  const iv = generateRandomBytes(12);

  const start = performance.now();
  for (let iter = 0; iter < iterations; iter++) {
    await crypto.subtle.encrypt(
      { name: 'AES-GCM', iv },
      key,
      data
    );
  }
  const end = performance.now();

  return {
    totalMs: end - start,
    avgMs: (end - start) / iterations,
    opsPerSec: iterations / ((end - start) / 1000)
  };
}

async function benchmarkChaCha8(dataSize, iterations) {
  const data = generateRandomBytes(dataSize);
  const key = generateRandomBytes(32);
  const nonce = generateRandomBytes(8);

  const start = performance.now();
  for (let iter = 0; iter < iterations; iter++) {
    chachaModule.encrypt_chacha8(data, key, nonce);
  }
  const end = performance.now();

  return {
    totalMs: end - start,
    avgMs: (end - start) / iterations,
    opsPerSec: iterations / ((end - start) / 1000)
  };
}

async function benchmarkChaCha12(dataSize, iterations) {
  const data = generateRandomBytes(dataSize);
  const key = generateRandomBytes(32);
  const nonce = generateRandomBytes(8);

  const start = performance.now();
  for (let iter = 0; iter < iterations; iter++) {
    chachaModule.encrypt_chacha12(data, key, nonce);
  }
  const end = performance.now();

  return {
    totalMs: end - start,
    avgMs: (end - start) / iterations,
    opsPerSec: iterations / ((end - start) / 1000)
  };
}

async function benchmarkChaCha20(dataSize, iterations) {
  const data = generateRandomBytes(dataSize);
  const key = generateRandomBytes(32);
  const nonce = generateRandomBytes(8);

  const start = performance.now();
  for (let iter = 0; iter < iterations; iter++) {
    chachaModule.encrypt(data, key, nonce);
  }
  const end = performance.now();

  return {
    totalMs: end - start,
    avgMs: (end - start) / iterations,
    opsPerSec: iterations / ((end - start) / 1000)
  };
}

function formatResult(name, result) {
  return `${name}: ${result.avgMs.toFixed(4)}ms avg, ${result.opsPerSec.toFixed(0)} ops/sec`;
}

async function runBenchmarks() {
  console.log('Initializing...');
  await initCrypto();
  await initChacha();

  console.log('\n===== WebCrypto vs ChaCha WASM Benchmark =====');
  console.log(`Environment: ${isBrowser ? 'Browser' : 'Node.js'}`);
  console.log(`Iterations per test: ${ITERATIONS}`);
  console.log('');

  for (const size of DATA_SIZES) {
    console.log(`\n--- Data Size: ${size} bytes ---`);

    const aesResult = await benchmarkAES256GCM(size, ITERATIONS);
    console.log(formatResult('AES-256-GCM', aesResult));

    const chacha8Result = await benchmarkChaCha8(size, ITERATIONS);
    console.log(formatResult('ChaCha8', chacha8Result));

    const chacha12Result = await benchmarkChaCha12(size, ITERATIONS);
    console.log(formatResult('ChaCha12', chacha12Result));

    const chacha20Result = await benchmarkChaCha20(size, ITERATIONS);
    console.log(formatResult('ChaCha20', chacha20Result));
  }

  console.log('\n===== Benchmark Complete =====');
}

runBenchmarks().catch(console.error);
