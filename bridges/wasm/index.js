/**
 * Voyager Observatory - WASM Bridge
 *
 * High-level JavaScript wrapper for the Voyager Observatory WASM kernel.
 * Enables "Planetarium Scans" in browser and Node.js environments.
 *
 * @example
 * const { VoyagerObservatory } = require('@voyager-observatory/wasm');
 *
 * const vo = new VoyagerObservatory();
 * const context = vo.scan([
 *   { path: 'src/main.rs', content: 'fn main() {}' },
 *   { path: 'README.md', content: '# Project' }
 * ], { lens: 'architecture' });
 */

const wasm = require('./pkg/pm_encoder.js');

/**
 * Voyager Observatory - Context Serialization Engine
 *
 * Transforms codebases into AI-digestible context with intelligent
 * token budgeting and semantic filtering.
 */
class VoyagerObservatory {
    constructor() {
        this._version = wasm.wasm_version();
        this._lenses = JSON.parse(wasm.wasm_get_lenses());
    }

    /**
     * Get the Voyager Observatory version
     * @returns {string} Version string (e.g., "1.0.0")
     */
    get version() {
        return this._version;
    }

    /**
     * Get available spectral filters (lenses)
     * @returns {string[]} Array of lens names
     */
    get lenses() {
        return this._lenses;
    }

    /**
     * Perform a Planetarium Scan - serialize files into AI context
     *
     * @param {Array<{path: string, content: string}>} files - Files to scan
     * @param {Object} [config={}] - Scan configuration
     * @param {string} [config.lens] - Spectral filter (architecture, security, debug, onboarding)
     * @param {number} [config.token_budget] - Maximum tokens for output
     * @param {string} [config.budget_strategy] - Strategy when over budget (drop, truncate, hybrid)
     * @param {number} [config.truncate_lines] - Max lines per file before truncation
     * @param {string} [config.truncate_mode] - Truncation mode (head, tail, smart)
     * @returns {string} Serialized context in Plus/Minus format
     *
     * @example
     * const context = vo.scan([
     *   { path: 'src/lib.rs', content: 'pub fn hello() {}' }
     * ], { lens: 'architecture', token_budget: 10000 });
     */
    scan(files, config = {}) {
        const filesJson = JSON.stringify(files);
        const configJson = JSON.stringify(config);
        return wasm.wasm_serialize(filesJson, configJson);
    }

    /**
     * Quick scan with onboarding lens - for rapid context generation
     *
     * @param {Array<{path: string, content: string}>} files - Files to scan
     * @returns {string} Onboarding-focused context
     */
    quickScan(files) {
        return this.scan(files, { lens: 'onboarding' });
    }

    /**
     * Onboarding scan - highlights getting started guides, examples
     *
     * @param {Array<{path: string, content: string}>} files - Files to scan
     * @param {number} [tokenBudget] - Optional token budget
     * @returns {string} Onboarding-focused context
     */
    onboardingScan(files, tokenBudget) {
        const config = { lens: 'onboarding' };
        if (tokenBudget) config.token_budget = tokenBudget;
        return this.scan(files, config);
    }

    /**
     * Architecture scan - highlights system design, entry points, configs
     *
     * @param {Array<{path: string, content: string}>} files - Files to scan
     * @param {number} [tokenBudget] - Optional token budget
     * @returns {string} Architecture-focused context
     */
    architectureScan(files, tokenBudget) {
        const config = { lens: 'architecture' };
        if (tokenBudget) config.token_budget = tokenBudget;
        return this.scan(files, config);
    }

    /**
     * Security scan - highlights auth, crypto, input validation
     *
     * @param {Array<{path: string, content: string}>} files - Files to scan
     * @param {number} [tokenBudget] - Optional token budget
     * @returns {string} Security-focused context
     */
    securityScan(files, tokenBudget) {
        const config = { lens: 'security' };
        if (tokenBudget) config.token_budget = tokenBudget;
        return this.scan(files, config);
    }

    /**
     * Debug scan - highlights tests, error handlers, logs
     *
     * @param {Array<{path: string, content: string}>} files - Files to scan
     * @param {number} [tokenBudget] - Optional token budget
     * @returns {string} Debug-focused context
     */
    debugScan(files, tokenBudget) {
        const config = { lens: 'debug' };
        if (tokenBudget) config.token_budget = tokenBudget;
        return this.scan(files, config);
    }
}

/**
 * Create a new Voyager Observatory instance
 * @returns {VoyagerObservatory}
 */
function createObservatory() {
    return new VoyagerObservatory();
}

module.exports = {
    VoyagerObservatory,
    createObservatory,
    // Re-export raw WASM functions for advanced usage
    wasm_serialize: wasm.wasm_serialize,
    wasm_version: wasm.wasm_version,
    wasm_get_lenses: wasm.wasm_get_lenses,
};
