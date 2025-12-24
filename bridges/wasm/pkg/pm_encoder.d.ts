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
