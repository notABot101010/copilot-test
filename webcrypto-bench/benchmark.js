// WebCrypto vs ChaCha WASM Benchmark
// Benchmarks AES-256-GCM (WebCrypto) against ChaCha8/12/20 (WASM)

const DATA_SIZES = [
  64,
  256,
  1024,
  4096,
  16384,
  65536,
  1024 * 1024,      // 1 MiB
  5 * 1024 * 1024,  // 5 MiB
  10 * 1024 * 1024  // 10 MiB
];
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
  // getRandomValues has a 65536 byte limit, so fill in chunks
  const chunkSize = 65536;
  for (let offset = 0; offset < length; offset += chunkSize) {
    const chunk = buffer.subarray(offset, Math.min(offset + chunkSize, length));
    crypto.getRandomValues(chunk);
  }
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

  const totalMs = end - start;
  const totalBytes = dataSize * iterations;
  const throughputMBps = (totalBytes / (totalMs / 1000)) / (1024 * 1024);

  return {
    totalMs,
    avgMs: totalMs / iterations,
    opsPerSec: iterations / (totalMs / 1000),
    throughputMBps
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

  const totalMs = end - start;
  const totalBytes = dataSize * iterations;
  const throughputMBps = (totalBytes / (totalMs / 1000)) / (1024 * 1024);

  return {
    totalMs,
    avgMs: totalMs / iterations,
    opsPerSec: iterations / (totalMs / 1000),
    throughputMBps
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

  const totalMs = end - start;
  const totalBytes = dataSize * iterations;
  const throughputMBps = (totalBytes / (totalMs / 1000)) / (1024 * 1024);

  return {
    totalMs,
    avgMs: totalMs / iterations,
    opsPerSec: iterations / (totalMs / 1000),
    throughputMBps
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

  const totalMs = end - start;
  const totalBytes = dataSize * iterations;
  const throughputMBps = (totalBytes / (totalMs / 1000)) / (1024 * 1024);

  return {
    totalMs,
    avgMs: totalMs / iterations,
    opsPerSec: iterations / (totalMs / 1000),
    throughputMBps
  };
}

function formatResult(name, result) {
  return `${name}: ${result.avgMs.toFixed(4)}ms avg, ${result.opsPerSec.toFixed(0)} ops/sec, ${result.throughputMBps.toFixed(2)} MB/s`;
}

function formatDataSize(bytes) {
  if (bytes >= 1024 * 1024) {
    return `${(bytes / (1024 * 1024)).toFixed(0)} MiB`;
  } else if (bytes >= 1024) {
    return `${(bytes / 1024).toFixed(0)} KiB`;
  }
  return `${bytes} bytes`;
}

function getIterationsForSize(size) {
  // Reduce iterations for large data sizes to keep benchmark time reasonable
  if (size >= 10 * 1024 * 1024) return 10;   // 10 MiB
  if (size >= 5 * 1024 * 1024) return 20;    // 5 MiB
  if (size >= 1024 * 1024) return 100;       // 1 MiB
  return ITERATIONS;
}

async function runBenchmarks() {
  console.log('Initializing...');
  await initCrypto();
  await initChacha();

  console.log('\n===== WebCrypto vs ChaCha WASM Benchmark =====');
  console.log(`Environment: ${isBrowser ? 'Browser' : 'Node.js'}`);
  console.log('');

  for (const size of DATA_SIZES) {
    const iterations = getIterationsForSize(size);
    console.log(`\n--- Data Size: ${formatDataSize(size)} (${iterations} iterations) ---`);

    const aesResult = await benchmarkAES256GCM(size, iterations);
    console.log(formatResult('AES-256-GCM', aesResult));

    const chacha8Result = await benchmarkChaCha8(size, iterations);
    console.log(formatResult('ChaCha8', chacha8Result));

    const chacha12Result = await benchmarkChaCha12(size, iterations);
    console.log(formatResult('ChaCha12', chacha12Result));

    const chacha20Result = await benchmarkChaCha20(size, iterations);
    console.log(formatResult('ChaCha20', chacha20Result));
  }

  console.log('\n===== Benchmark Complete =====');
}

runBenchmarks().catch(console.error);
