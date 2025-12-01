/* tslint:disable */
/* eslint-disable */

export class WasmBindgenTestContext {
  free(): void;
  [Symbol.dispose](): void;
  /**
   * Handle filter argument.
   */
  filtered_count(filtered: number): void;
  /**
   * Handle `--include-ignored` flag.
   */
  include_ignored(include_ignored: boolean): void;
  /**
   * Creates a new context ready to run tests.
   *
   * A `Context` is the main structure through which test execution is
   * coordinated, and this will collect output and results for all executed
   * tests.
   */
  constructor(is_bench: boolean);
  /**
   * Executes a list of tests, returning a promise representing their
   * eventual completion.
   *
   * This is the main entry point for executing tests. All the tests passed
   * in are the JS `Function` object that was plucked off the
   * `WebAssembly.Instance` exports list.
   *
   * The promise returned resolves to either `true` if all tests passed or
   * `false` if at least one test failed.
   */
  run(tests: any[]): Promise<any>;
}

/**
 * Used to read benchmark data, and then the runner stores it on the local disk.
 */
export function __wbgbench_dump(): Uint8Array | undefined;

/**
 * Used to write previous benchmark data before the benchmark, for later comparison.
 */
export function __wbgbench_import(baseline: Uint8Array): void;

/**
 * Handler for `console.debug` invocations. See above.
 */
export function __wbgtest_console_debug(args: Array<any>): void;

/**
 * Handler for `console.error` invocations. See above.
 */
export function __wbgtest_console_error(args: Array<any>): void;

/**
 * Handler for `console.info` invocations. See above.
 */
export function __wbgtest_console_info(args: Array<any>): void;

/**
 * Handler for `console.log` invocations.
 *
 * If a test is currently running it takes the `args` array and stringifies
 * it and appends it to the current output of the test. Otherwise it passes
 * the arguments to the original `console.log` function, psased as
 * `original`.
 */
export function __wbgtest_console_log(args: Array<any>): void;

/**
 * Handler for `console.warn` invocations. See above.
 */
export function __wbgtest_console_warn(args: Array<any>): void;

export function __wbgtest_cov_dump(): Uint8Array | undefined;

/**
 * Clear all MLS state (for logout)
 */
export function clear_state(): void;

/**
 * Create a new MLS group
 */
export function create_group(group_id: string): void;

/**
 * Create an invitation for a new member
 */
export function create_invite(group_id: string, invitee_key_package_b64: string): any;

/**
 * Decrypt a message from the group
 */
export function decrypt_message(group_id: string, ciphertext_b64: string): string;

/**
 * Encrypt a message for the group
 */
export function encrypt_message(group_id: string, plaintext: string): string;

/**
 * Generate key packages for the client
 */
export function generate_key_packages(count: number): any;

/**
 * Check if we have group state for a given group ID
 */
export function has_group_state(group_id: string): boolean;

/**
 * Initialize the MLS client with a username
 */
export function init_mls(username: string): void;

/**
 * Process a commit message to update group state
 */
export function process_commit(group_id: string, commit_b64: string): void;

/**
 * Process a welcome message to join a group
 */
export function process_welcome(welcome_b64: string): string;

/**
 * Save state to localStorage
 */
export function save_state_to_storage(username: string): void;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly clear_state: () => void;
  readonly create_group: (a: number, b: number) => [number, number];
  readonly create_invite: (a: number, b: number, c: number, d: number) => [number, number, number];
  readonly decrypt_message: (a: number, b: number, c: number, d: number) => [number, number, number, number];
  readonly encrypt_message: (a: number, b: number, c: number, d: number) => [number, number, number, number];
  readonly generate_key_packages: (a: number) => [number, number, number];
  readonly has_group_state: (a: number, b: number) => number;
  readonly init_mls: (a: number, b: number) => [number, number];
  readonly process_commit: (a: number, b: number, c: number, d: number) => [number, number];
  readonly process_welcome: (a: number, b: number) => [number, number, number, number];
  readonly save_state_to_storage: (a: number, b: number) => [number, number];
  readonly __wbg_wasmbindgentestcontext_free: (a: number, b: number) => void;
  readonly __wbgtest_console_debug: (a: any) => void;
  readonly __wbgtest_console_error: (a: any) => void;
  readonly __wbgtest_console_info: (a: any) => void;
  readonly __wbgtest_console_log: (a: any) => void;
  readonly __wbgtest_console_warn: (a: any) => void;
  readonly wasmbindgentestcontext_filtered_count: (a: number, b: number) => void;
  readonly wasmbindgentestcontext_include_ignored: (a: number, b: number) => void;
  readonly wasmbindgentestcontext_new: (a: number) => number;
  readonly wasmbindgentestcontext_run: (a: number, b: number, c: number) => any;
  readonly __wbgbench_dump: () => [number, number];
  readonly __wbgbench_import: (a: number, b: number) => void;
  readonly __wbgtest_cov_dump: () => [number, number];
  readonly wasm_bindgen__convert__closures_____invoke__h4319133022181043: (a: number, b: number, c: any) => void;
  readonly wasm_bindgen__closure__destroy__h995dd7520031f68d: (a: number, b: number) => void;
  readonly wasm_bindgen__convert__closures_____invoke__h255a89ae68d7021b: (a: number, b: number, c: any, d: number, e: any) => void;
  readonly wasm_bindgen__convert__closures_____invoke__h1c2f29fb06bfab95: (a: number, b: number, c: any, d: any) => void;
  readonly __externref_table_alloc: () => number;
  readonly __wbindgen_externrefs: WebAssembly.Table;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_exn_store: (a: number) => void;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __externref_table_dealloc: (a: number) => void;
  readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
*
* @returns {InitOutput}
*/
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
