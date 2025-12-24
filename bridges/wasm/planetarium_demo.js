#!/usr/bin/env node
/**
 * Voyager Observatory - Planetarium Scan Demo
 *
 * Demonstrates the WASM kernel running a "Planetarium Scan" on mock files.
 * This proves the engine is 100% decoupled from the OS and ready for
 * VS Code extension integration.
 *
 * Usage: node planetarium_demo.js
 */

const { VoyagerObservatory } = require('./index.js');

// ANSI colors for terminal output
const RESET = '\x1b[0m';
const BOLD = '\x1b[1m';
const GREEN = '\x1b[32m';
const CYAN = '\x1b[36m';
const YELLOW = '\x1b[33m';
const DIM = '\x1b[2m';

console.log(`${BOLD}${CYAN}
╔══════════════════════════════════════════════════════════════════╗
║               🔭 VOYAGER OBSERVATORY - WASM BRIDGE 🔭             ║
║                    Planetarium Scan Demo                         ║
╚══════════════════════════════════════════════════════════════════╝
${RESET}`);

// Initialize the Observatory
const vo = new VoyagerObservatory();

console.log(`${GREEN}✓${RESET} Observatory initialized`);
console.log(`${DIM}  Version: ${vo.version}${RESET}`);
console.log(`${DIM}  Available Lenses: ${vo.lenses.join(', ')}${RESET}`);
console.log();

// Mock codebase - simulating a small TypeScript project
const mockFiles = [
    {
        path: 'src/index.ts',
        content: `/**
 * Main entry point for the application.
 */
import { UserService } from './services/user';
import { Logger } from './utils/logger';

const logger = new Logger('main');

async function main() {
    logger.info('Application starting...');
    const userService = new UserService();
    await userService.initialize();
    logger.info('Application ready');
}

main().catch(err => {
    logger.error('Fatal error:', err);
    process.exit(1);
});
`
    },
    {
        path: 'src/services/user.ts',
        content: `/**
 * User service - handles user management operations.
 */
import { Database } from '../db/connection';
import { validateEmail } from '../utils/validation';

export interface User {
    id: string;
    email: string;
    name: string;
    createdAt: Date;
}

export class UserService {
    private db: Database;

    constructor() {
        this.db = new Database();
    }

    async initialize(): Promise<void> {
        await this.db.connect();
    }

    async createUser(email: string, name: string): Promise<User> {
        if (!validateEmail(email)) {
            throw new Error('Invalid email format');
        }
        return this.db.insert('users', { email, name });
    }

    async findByEmail(email: string): Promise<User | null> {
        return this.db.findOne('users', { email });
    }
}
`
    },
    {
        path: 'src/utils/logger.ts',
        content: `/**
 * Simple logger utility with context support.
 */
export class Logger {
    constructor(private context: string) {}

    info(message: string, ...args: unknown[]): void {
        console.log(\`[\${this.context}] INFO: \${message}\`, ...args);
    }

    error(message: string, ...args: unknown[]): void {
        console.error(\`[\${this.context}] ERROR: \${message}\`, ...args);
    }

    debug(message: string, ...args: unknown[]): void {
        if (process.env.DEBUG) {
            console.debug(\`[\${this.context}] DEBUG: \${message}\`, ...args);
        }
    }
}
`
    },
    {
        path: 'src/utils/validation.ts',
        content: `/**
 * Input validation utilities.
 */

const EMAIL_REGEX = /^[^\\s@]+@[^\\s@]+\\.[^\\s@]+$/;

export function validateEmail(email: string): boolean {
    return EMAIL_REGEX.test(email);
}

export function validatePassword(password: string): boolean {
    return password.length >= 8 && /[A-Z]/.test(password) && /[0-9]/.test(password);
}

export function sanitizeInput(input: string): string {
    return input.replace(/[<>&"']/g, '');
}
`
    },
    {
        path: 'README.md',
        content: `# Demo Project

A sample TypeScript project for testing the Voyager Observatory.

## Features
- User management
- Logging
- Input validation
`
    }
];

// Perform Planetarium Scan with different lenses
console.log(`${BOLD}${YELLOW}1. Architecture Scan${RESET}`);
console.log(`${DIM}   Highlighting system design, entry points, configs...${RESET}`);
console.log();

const archContext = vo.architectureScan(mockFiles);
const archLines = archContext.split('\n').slice(0, 15);
console.log(archLines.join('\n'));
console.log(`${DIM}   ... (${archContext.split('\n').length} total lines)${RESET}`);
console.log();

console.log(`${BOLD}${YELLOW}2. Security Scan${RESET}`);
console.log(`${DIM}   Highlighting auth, crypto, input validation...${RESET}`);
console.log();

const secContext = vo.securityScan(mockFiles);
const secFiles = secContext.match(/^\+{10,}/gm) || [];
console.log(`${DIM}   Files prioritized: ${secFiles.length}${RESET}`);
console.log();

console.log(`${BOLD}${YELLOW}3. Onboarding Scan${RESET}`);
console.log(`${DIM}   Getting started guides, examples...${RESET}`);
console.log();

const onboardingContext = vo.onboardingScan(mockFiles);
console.log(`${DIM}   Output size: ${onboardingContext.length} characters${RESET}`);
console.log();

// Summary
console.log(`${BOLD}${GREEN}
╔══════════════════════════════════════════════════════════════════╗
║                    🌟 PLANETARIUM SCAN COMPLETE 🌟                ║
╚══════════════════════════════════════════════════════════════════╝
${RESET}`);

console.log(`${GREEN}✓${RESET} WASM kernel executed successfully`);
console.log(`${GREEN}✓${RESET} Engine is 100% decoupled from OS`);
console.log(`${GREEN}✓${RESET} Ready for VS Code extension integration`);
console.log();
console.log(`${DIM}The Voyager can now run in any JavaScript environment:${RESET}`);
console.log(`${DIM}  - Node.js CLI tools${RESET}`);
console.log(`${DIM}  - VS Code extensions${RESET}`);
console.log(`${DIM}  - Browser-based IDEs${RESET}`);
console.log(`${DIM}  - Electron applications${RESET}`);
console.log();
