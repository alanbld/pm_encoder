#!/usr/bin/env node
/**
 * WASM Test Script for pm_encoder
 *
 * Prerequisites:
 *   cargo build --target wasm32-unknown-unknown --features wasm --lib --release
 *
 * Run:
 *   node test_wasm.mjs
 */

import { readFile } from 'fs/promises';
import { WASI } from 'wasi';

// Note: For full wasm-bindgen support, use wasm-pack instead
// This is a basic test to verify the WASM compiles and loads

async function main() {
    console.log('Loading WASM module...');

    const wasmPath = './target/wasm32-unknown-unknown/release/pm_encoder.wasm';

    try {
        const wasmBuffer = await readFile(wasmPath);
        console.log(`WASM file size: ${(wasmBuffer.length / 1024 / 1024).toFixed(2)} MB`);

        // Basic validation - check it's a valid WASM module
        const magic = wasmBuffer.slice(0, 4);
        if (magic[0] === 0x00 && magic[1] === 0x61 && magic[2] === 0x73 && magic[3] === 0x6d) {
            console.log('Valid WASM magic number detected');
        }

        // Try to compile (doesn't execute, just validates)
        const module = await WebAssembly.compile(wasmBuffer);
        console.log('WASM module compiled successfully!');

        // List exports
        const exports = WebAssembly.Module.exports(module);
        console.log(`\nExported functions (${exports.length}):`);
        exports.slice(0, 20).forEach(exp => {
            console.log(`  - ${exp.name} (${exp.kind})`);
        });
        if (exports.length > 20) {
            console.log(`  ... and ${exports.length - 20} more`);
        }

        console.log('\n✅ WASM module is valid and ready for wasm-bindgen integration');
        console.log('\nNext steps:');
        console.log('  1. Install wasm-pack: cargo install wasm-pack');
        console.log('  2. Build with bindings: wasm-pack build --target web --features wasm');
        console.log('  3. Use generated pkg/ directory in your web app');

    } catch (error) {
        console.error('Error:', error.message);
        process.exit(1);
    }
}

main();
