/* tslint:disable */
/* eslint-disable */

/**
 * Get available lens names (WASM)
 */
export function wasm_get_lenses(): string;

/**
 * Serialize files to Plus/Minus format (WASM entry point)
 *
 * # Arguments
 * * `json_files` - JSON array of {path, content} objects
 * * `json_config` - Optional JSON config object
 *
 * # Returns
 * * Serialized context string or error
 *
 * # Example (JavaScript)
 * ```javascript
 * const files = [
 *   { path: "main.py", content: "print('hello')" },
 *   { path: "lib.py", content: "def helper(): pass" }
 * ];
 * const config = { lens: "architecture", token_budget: 100000 };
 * const context = wasm_serialize(JSON.stringify(files), JSON.stringify(config));
 * ```
 */
export function wasm_serialize(json_files: string, json_config: string): string;

/**
 * Get library version (WASM)
 */
export function wasm_version(): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly wasm_get_lenses: () => [number, number];
  readonly wasm_serialize: (a: number, b: number, c: number, d: number) => [number, number, number, number];
  readonly wasm_version: () => [number, number];
  readonly __wbindgen_externrefs: WebAssembly.Table;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __externref_table_dealloc: (a: number) => void;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
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
