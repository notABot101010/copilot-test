// Hash Function WASM Benchmark
// Benchmarks BLAKE3, SHA-256, and SHA-512 compiled to WebAssembly

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
let hashModule;

async function initCrypto() {
  if (isBrowser) {
    crypto = window.crypto;
  } else if (isNode) {
    crypto = (await import('node:crypto')).webcrypto;
  }
}

async function initHashModule() {
  if (isBrowser) {
    const init = (await import('./pkg/wasm_hash_bench.js')).default;
    hashModule = await init();
  } else if (isNode) {
    const { readFile } = await import('node:fs/promises');
    const { fileURLToPath } = await import('node:url');
    const { dirname, join } = await import('node:path');

    const currentDir = dirname(fileURLToPath(import.meta.url));
    const wasmPath = join(currentDir, 'pkg', 'wasm_hash_bench_bg.wasm');
    const wasmBuffer = await readFile(wasmPath);

    const wasmModule = await import('./pkg/wasm_hash_bench.js');
    await wasmModule.default({ module_or_path: wasmBuffer });
    hashModule = wasmModule;
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

async function benchmarkBlake3(dataSize, iterations) {
  const data = generateRandomBytes(dataSize);

  const start = performance.now();
  for (let iter = 0; iter < iterations; iter++) {
    hashModule.hash_blake3(data);
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

async function benchmarkSha256(dataSize, iterations) {
  const data = generateRandomBytes(dataSize);

  const start = performance.now();
  for (let iter = 0; iter < iterations; iter++) {
    hashModule.hash_sha256(data);
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

async function benchmarkSha512(dataSize, iterations) {
  const data = generateRandomBytes(dataSize);

  const start = performance.now();
  for (let iter = 0; iter < iterations; iter++) {
    hashModule.hash_sha512(data);
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
  await initHashModule();

  console.log('\n===== Hash Function WASM Benchmark =====');
  console.log(`Environment: ${isBrowser ? 'Browser' : 'Node.js'}`);
  console.log('');

  for (const size of DATA_SIZES) {
    const iterations = getIterationsForSize(size);
    console.log(`\n--- Data Size: ${formatDataSize(size)} (${iterations} iterations) ---`);

    const blake3Result = await benchmarkBlake3(size, iterations);
    console.log(formatResult('BLAKE3', blake3Result));

    const sha256Result = await benchmarkSha256(size, iterations);
    console.log(formatResult('SHA-256', sha256Result));

    const sha512Result = await benchmarkSha512(size, iterations);
    console.log(formatResult('SHA-512', sha512Result));

    // Calculate relative performance

    const results = [
      { name: 'BLAKE3', throughput: blake3Result.throughputMBps },
      { name: 'SHA-256', throughput: sha256Result.throughputMBps },
      { name: 'SHA-512', throughput: sha512Result.throughputMBps }
    ];
    const fastest = results.reduce((max, curr) => curr.throughput > max.throughput ? curr : max);
    console.log(`Fastest: ${fastest.name}`);
  }

  console.log('\n===== Benchmark Complete =====');
}

runBenchmarks().catch(console.error);
