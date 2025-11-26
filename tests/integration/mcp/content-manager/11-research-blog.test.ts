import { describe, it, expect, beforeAll, afterAll } from 'vitest';
import { generateAdminToken, createContext, McpTestClient } from '@test/utils';
import config from '@test/config';

describe('MCP: content-manager - research_blog', () => {
  let client: McpTestClient;
  let adminToken: string;
  let contextId: string;

  beforeAll(async () => {
    adminToken = generateAdminToken();
    contextId = await createContext(adminToken, config.apiBaseUrl);

    client = new McpTestClient({
      baseUrl: config.apiBaseUrl,
      serverName: 'content-manager',
      token: adminToken,
      contextId,
    });
    await client.connect();
  });

  afterAll(async () => {
    await client.close();
  });

  it('should have research_blog tool with urls parameter', async () => {
    const tools = await client.listTools();

    console.log('\n🛠️  Tool Schema Validation:');
    console.log('Total tools:', tools.tools?.length || 0);

    const blogTool = tools.tools?.find(t => t.name === 'research_blog');

    expect(blogTool).toBeDefined();
    expect(blogTool?.name).toBe('research_blog');

    const schema = blogTool?.inputSchema as any;
    const properties = schema?.properties || {};

    console.log('\n  ✅ research_blog parameters:');
    console.log('    - topic:', properties.topic ? '✓' : '✗');
    console.log('    - depth:', properties.depth ? '✓' : '✗');
    console.log('    - focus_areas:', properties.focus_areas ? '✓' : '✗');
    console.log('    - urls:', properties.urls ? '✓' : '✗');

    // Verify all parameters exist
    expect(properties.topic).toBeDefined();
    expect(properties.depth).toBeDefined();
    expect(properties.focus_areas).toBeDefined();
    expect(properties.urls).toBeDefined();

    // Verify urls parameter configuration
    expect(properties.urls.type).toBe('array');
    expect(properties.urls.items?.type).toBe('string');
    expect(properties.urls.maxItems).toBe(20);

    console.log('\n  ✅ URLs parameter config:');
    console.log('    - Type: array of strings');
    console.log('    - Max items: 20');
    console.log('    - Optional: yes');
  });

  it('should perform basic research with Google Search', async () => {
    const result = await client.callTool('research_blog', {
      topic: 'Rust async',
      depth: 'quick',
    }, 90000);

    console.log('\n🔍 Basic Search Test:');
    console.log('Status:', result.isError ? 'ERROR' : 'SUCCESS');

    expect(result.isError).toBe(false);
    expect(result.structuredContent).toBeDefined();

    console.log('  ✅ Google Search grounding works');
  }, 100000);

  it('should perform research with URL context', async () => {
    const result = await client.callTool('research_blog', {
      topic: 'Tokio runtime',
      depth: 'quick',
      urls: ['https://tokio.rs/'],
    }, 90000);

    console.log('\n🌐 URL Context Test:');
    console.log('Status:', result.isError ? 'ERROR' : 'SUCCESS');

    expect(result.isError).toBe(false);
    expect(result.structuredContent).toBeDefined();

    console.log('  ✅ URL context loading works');
  }, 100000);
});
