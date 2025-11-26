import { beforeAll, afterAll } from 'vitest';

/**
 * Global vitest setup - Runs once before all tests
 * MCP integration tests connect to running server via HTTP
 */

beforeAll(async () => {
  console.log('');
  console.log('═══════════════════════════════════════════════════════════');
  console.log('🧪 MCP INTEGRATION TEST SETUP');
  console.log('═══════════════════════════════════════════════════════════');
  console.log('');
  console.log('Base URL:', process.env.BASE_URL || 'http://localhost:8080');
  console.log('JWT Secret:', process.env.JWT_SECRET ? 'Set' : 'Using default');
  console.log('');
  console.log('Prerequisites:');
  console.log('  1. API server running: just start');
  console.log('  2. MCP servers enabled in config');
  console.log('');
  console.log('═══════════════════════════════════════════════════════════');
  console.log('🚀 Starting tests...');
  console.log('═══════════════════════════════════════════════════════════');
  console.log('');
});

/**
 * Cleanup after all tests
 */
afterAll(async () => {
  console.log('');
  console.log('═══════════════════════════════════════════════════════════');
  console.log('✅ MCP TEST SUITE COMPLETE');
  console.log('═══════════════════════════════════════════════════════════');
  console.log('');
});
