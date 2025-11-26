import path from 'path';

/**
 * Integration Test Configuration
 *
 * RUNNING TESTS:
 *
 * ✅ CORRECT: Run from /core/tests/integration directory
 *   cd core/tests/integration
 *   npx vitest run mcp/content-manager/02-reply-to-twitter.test.ts
 *
 * ❌ INCORRECT: Do NOT run from /core directory
 *   This will cause module resolution issues for @test/* aliases
 *
 * PREREQUISITES:
 *   1. Start API server: cd /core && just api
 *   2. Keep API running while tests execute
 *
 * MODULE ALIASES:
 *   @test/utils   -> ./shared/utils
 *   @test/config  -> ./shared/config.ts
 *   @test/types   -> ./shared/types
 */

export const config = {
  // API Configuration
  apiBaseUrl: process.env.BASE_URL || 'http://localhost:8080',

  // Database Configuration
  databasePath: process.env.DATABASE_URL || '../../database/systemprompt.db',
  databaseUrl: `sqlite://${process.env.DATABASE_URL || '../../database/systemprompt.db'}`,

  // Auth Configuration
  adminToken: process.env.ADMIN_TOKEN || '',
  jwtSecret: process.env.JWT_SECRET || 'your-very-secure-jwt-secret-key-here',
  jwtIssuer: process.env.JWT_ISSUER || 'systemprompt-os',

  // Test Configuration
  testTimeoutMs: 30000,
  testRetries: 2,

  // Feature Flags
  enableSSE: true,
  enableA2A: true,
  enableMCP: true,
};

export default config;
