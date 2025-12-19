#!/usr/bin/env node
/**
 * Full WASM Test for pm_encoder
 *
 * Prerequisites:
 *   wasm-pack build --target web --features wasm
 *
 * Run:
 *   node test_wasm_full.mjs
 */

import { readFile } from 'fs/promises';

// Load the WASM module
const wasmPath = './pkg/pm_encoder_bg.wasm';
const jsPath = './pkg/pm_encoder.js';

async function main() {
    console.log('=== pm_encoder WASM Test ===\n');

    // Dynamic import of the generated JS
    const wasmModule = await import(jsPath);

    // Initialize WASM
    const wasmBytes = await readFile(wasmPath);
    await wasmModule.default(wasmBytes);

    // Test 1: Version
    console.log('1. Version:');
    const version = wasmModule.wasm_version();
    console.log(`   ${version}\n`);

    // Test 2: Available lenses
    console.log('2. Available Lenses:');
    const lenses = JSON.parse(wasmModule.wasm_get_lenses());
    console.log(`   ${lenses.join(', ')}\n`);

    // Test 3: Serialize files
    console.log('3. Serialize Test Files:');
    const files = [
        { path: "src/main.py", content: "#!/usr/bin/env python3\nimport os\n\ndef main():\n    print('Hello, World!')\n\nif __name__ == '__main__':\n    main()\n" },
        { path: "src/utils.py", content: "def helper():\n    return 42\n" },
        { path: "README.md", content: "# Test Project\n\nThis is a test.\n" }
    ];

    const config = {};  // Default config

    const context = wasmModule.wasm_serialize(
        JSON.stringify(files),
        JSON.stringify(config)
    );

    console.log('   Input: 3 files');
    console.log(`   Output: ${context.length} characters\n`);
    console.log('--- Output Preview (first 500 chars) ---');
    console.log(context.substring(0, 500));
    console.log('...\n');

    // Test 4: With lens
    console.log('4. Serialize with Architecture Lens:');
    const configWithLens = { lens: "architecture" };

    const contextWithLens = wasmModule.wasm_serialize(
        JSON.stringify(files),
        JSON.stringify(configWithLens)
    );

    console.log(`   Output: ${contextWithLens.length} characters`);
    console.log(`   (Lens may add metadata file)\n`);

    console.log('=== All Tests Passed ===');
}

main().catch(err => {
    console.error('Error:', err);
    process.exit(1);
});
