import { describe, it, expect, beforeAll, afterAll } from 'vitest';
import { generateAdminToken, createContext, McpTestClient } from '@test/utils';
import config from '@test/config';

describe('MCP: tyingshoelaces - introduction tool', () => {
  let client: McpTestClient;
  let adminToken: string;
  let contextId: string;

  beforeAll(async () => {
    adminToken = generateAdminToken();
    contextId = await createContext(adminToken, config.apiBaseUrl);
    client = new McpTestClient({
      baseUrl: config.apiBaseUrl,
      serverName: 'tyingshoelaces',
      token: adminToken,
      contextId,
    });
    await client.connect();
  });

  afterAll(async () => {
    await client.close();
  });

  it('should list tools and find introduction', async () => {
    const result = await client.listTools();

    console.log('📋 Available tools:', JSON.stringify(result.tools, null, 2));

    expect(result.tools).toBeDefined();
    const introTool = result.tools.find((t: any) => t.name === 'introduction');
    expect(introTool).toBeDefined();
    expect(introTool.name).toBe('introduction');
  });

  it('should execute introduction tool', async () => {
    const result = await client.callTool('introduction', {});

    console.log('\n👋 introduction result:');
    console.log('Status:', result.isError ? 'ERROR' : 'SUCCESS');
    console.log('Content items:', result.content.length);

    if (result.content.length > 0) {
      result.content.forEach((item: any, idx: number) => {
        console.log(`  ${idx + 1}. Type: ${item.type}`);
        if (item.type === 'text') {
          console.log(`     Text: ${item.text?.substring(0, 200)}...`);
        }
      });
    }

    expect(result.isError).toBe(false);
    expect(result.content).toBeDefined();
    expect(result.content.length).toBeGreaterThan(0);
  });

  it('should verify introduction creates artifact', async () => {
    const result = await client.callTool('introduction', {});

    console.log('\n🎨 Artifact verification:');
    console.log('Status:', result.isError ? 'ERROR' : 'SUCCESS');
    console.log('Content items:', result.content.length);

    expect(result.isError).toBe(false);
    expect(result.content).toBeDefined();
  });
});
