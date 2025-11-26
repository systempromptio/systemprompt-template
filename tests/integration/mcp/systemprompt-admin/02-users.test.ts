import { describe, it, expect, beforeAll, afterAll } from 'vitest';
import { generateAdminToken, createContext, McpTestClient } from '@test/utils';
import config from '@test/config';

describe('MCP: systemprompt-admin - users tool', () => {
  let client: McpTestClient;
  let adminToken: string;
  let contextId: string;
  let testUserId: string;

  beforeAll(async () => {
    adminToken = generateAdminToken();
    contextId = await createContext(adminToken, config.apiBaseUrl);

    client = new McpTestClient({
      baseUrl: config.apiBaseUrl,
      serverName: 'systemprompt-admin',
      token: adminToken,
      contextId,
    });
    await client.connect();
  });

  afterAll(async () => {
    await client.close();
  });

  it('should list all users', async () => {
    const result = await client.callTool('user', {});

    console.log('\n👥 Users list result:');
    console.log('Status:', result.isError ? 'ERROR' : 'SUCCESS');

    if (result.isError) {
      console.log('Error:', JSON.stringify(result, null, 2));
    } else {
      if (result.structuredContent) {
        const response = result.structuredContent as any;
        console.log('  Total users:', response.count);
        console.log('  Users found:', response.items?.length || 0);

        if (response.items && response.items.length > 0) {
          const firstUser = response.items[0];
          testUserId = firstUser.id;
          console.log('\n  📝 Sample user:');
          console.log('    ID:', firstUser.id);
          console.log('    Name:', firstUser.name);
          console.log('    Email:', firstUser.email);
          console.log('    Roles:', firstUser.roles);
          console.log('    Sessions:', firstUser.total_sessions);
        }
      }
    }

    expect(result.isError).toBe(false);
    expect(result.structuredContent).toBeDefined();

    const response = result.structuredContent as any;
    expect(response.items).toBeDefined();
    expect(Array.isArray(response.items)).toBe(true);
    expect(response.count).toBeGreaterThanOrEqual(0);
  });

  it('should filter users by user_id', async () => {
    if (!testUserId) {
      console.log('⚠️  Skipping: No test user ID available');
      return;
    }

    const result = await client.callTool('user', {
      user_id: testUserId
    });

    console.log('\n👤 Filtered users result:');
    console.log('Status:', result.isError ? 'ERROR' : 'SUCCESS');

    if (result.structuredContent) {
      const response = result.structuredContent as any;
      console.log('  Total users:', response.count);
      console.log('  Users found:', response.items?.length || 0);

      if (response.items && response.items.length > 0) {
        const user = response.items[0];
        console.log('\n  📝 User details:');
        console.log('    ID:', user.id);
        console.log('    Name:', user.name);
        console.log('    Email:', user.email);
        console.log('    Roles:', user.roles);
        console.log('    Sessions:', user.total_sessions);
      }
    }

    expect(result.isError).toBe(false);
    expect(result.structuredContent).toBeDefined();

    const response = result.structuredContent as any;
    expect(response.items).toBeDefined();
    expect(response.items.length).toBeLessThanOrEqual(1);
    if (response.items.length > 0) {
      expect(response.items[0].id).toBe(testUserId);
    }
  });
});
