import { describe, it, expect } from 'vitest';
import { generateAdminToken } from './shared/utils/auth-utils';
import { ApiClient } from './shared/utils/api-client';
import { DatabaseConnection } from './shared/utils/db/connection';

describe('Diagnostic: Context Creation', () => {
  it('should create context and verify in database', async () => {
    const adminToken = generateAdminToken();
    const apiClient = new ApiClient(adminToken);
    const db = DatabaseConnection.getInstance();

    // Create context via API
    const response = await apiClient.createContext(`diagnostic-${Date.now()}`);
    console.log('API Response:', JSON.stringify(response, null, 2));

    expect(response.status).toBe(201);
    const contextId = response.data?.context_id;
    console.log('Created context_id:', contextId);

    // Verify in database
    const dbResult = db.prepare('SELECT * FROM user_contexts WHERE context_id = ?').get(contextId);
    console.log('DB Result:', dbResult);

    expect(dbResult).toBeDefined();
    expect(dbResult).toHaveProperty('context_id', contextId);
  });
});
