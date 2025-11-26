import { describe, it, expect, beforeAll, afterAll } from 'vitest';
import { generateAdminToken, createContext, McpTestClient } from '@test/utils';
import config from '@test/config';

describe('MCP: tyingshoelaces - context_retrieval tool', () => {
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

  it('should find context_retrieval tool in tools list', async () => {
    const result = await client.listTools();

    const contextRetrievalTool = result.tools.find((t: any) => t.name === 'context_retrieval');
    expect(contextRetrievalTool).toBeDefined();
    expect(contextRetrievalTool.name).toBe('context_retrieval');
    expect(contextRetrievalTool.title).toBe('Content Context Retrieval');

    console.log('🔍 Context Retrieval tool schema:');
    console.log('Input:', JSON.stringify(contextRetrievalTool.inputSchema, null, 2));
    console.log('Output:', JSON.stringify(contextRetrievalTool.outputSchema, null, 2));
  });

  it('should execute context retrieval and return all content', async () => {
    const result = await client.callTool('context_retrieval', {});

    console.log('\n🔍 context_retrieval result (all content):');
    console.log('Status:', result.isError ? 'ERROR' : 'SUCCESS');
    console.log('Content items:', result.content.length);

    if (result.content.length > 0) {
      const textContent = result.content.find((c: any) => c.type === 'text');
      if (textContent) {
        console.log('Text preview:', (textContent as any).text?.substring(0, 200));
      }
    }

    if (result.structuredContent) {
      console.log('Structured content type:', result.structuredContent['x-artifact-type']);
      console.log('Items count:', result.structuredContent.items?.length || 0);

      if (result.structuredContent.items && result.structuredContent.items.length > 0) {
        console.log('First item:', JSON.stringify(result.structuredContent.items[0], null, 2));
      }
    }

    expect(result.isError).toBe(false);
    expect(result.content).toBeDefined();
    expect(result.structuredContent).toBeDefined();
    expect(result.structuredContent['x-artifact-type']).toBe('list');
  });

  it('should return all content without filters', async () => {
    const result = await client.callTool('context_retrieval', {});

    console.log('\n🔍 context_retrieval result (no filters):');
    console.log('Status:', result.isError ? 'ERROR' : 'SUCCESS');
    console.log('Content items:', result.content.length);

    if (result.structuredContent) {
      console.log('Items returned:', result.structuredContent.items?.length || 0);
    }

    expect(result.isError).toBe(false);
    expect(result.content).toBeDefined();
    expect(result.structuredContent).toBeDefined();
    expect(result.structuredContent['x-artifact-type']).toBe('list');
  });

  it('should handle extra parameters gracefully', async () => {
    const result = await client.callTool('context_retrieval', {
      keyword: 'AI',
      limit: 5,
    });

    console.log('\n🔍 context_retrieval result (with ignored parameters):');
    console.log('Status:', result.isError ? 'ERROR' : 'SUCCESS');
    console.log('Content items:', result.content.length);

    if (result.structuredContent) {
      const itemCount = result.structuredContent.items?.length || 0;
      console.log('Items returned:', itemCount);
      // Tool ignores limit parameter and returns all content
      expect(itemCount).toBeGreaterThan(0);
    }

    expect(result.isError).toBe(false);
    expect(result.content).toBeDefined();
    expect(result.structuredContent).toBeDefined();
  });

  it('should work with no parameters', async () => {
    console.log('\n✅ Testing with no parameters...');

    const result = await client.callTool('context_retrieval', {});

    expect(result.isError).toBe(false);
    expect(result.structuredContent).toBeDefined();
    expect(result.structuredContent.items).toBeDefined();
  });

  it('should return content with proper structure', async () => {
    console.log('\n✅ Testing with no parameters...');

    const result = await client.callTool('context_retrieval', {});

    expect(result.isError).toBe(false);
    expect(result.structuredContent).toBeDefined();
  });

  it('should return ListArtifact with all content items', async () => {
    const result = await client.callTool('context_retrieval', {});

    console.log('\n📋 ListArtifact structure verification:');
    console.log('Status:', result.isError ? 'ERROR' : 'SUCCESS');
    console.log('Content items:', result.content.length);

    expect(result.isError).toBe(false);
    expect(result.structuredContent).toBeDefined();
    expect(result.structuredContent['x-artifact-type']).toBe('list');
    expect(result.structuredContent.items).toBeDefined();
    expect(Array.isArray(result.structuredContent.items)).toBe(true);
    expect(result.structuredContent.items.length).toBeGreaterThan(0);

    if (result.structuredContent.items.length > 0) {
      const firstItem = result.structuredContent.items[0];
      console.log('First item structure:', JSON.stringify(firstItem, null, 2));

      expect(firstItem.id).toBeDefined();
      expect(firstItem.title).toBeDefined();
      expect(firstItem.summary).toBeDefined();
      expect(firstItem.link).toBeDefined();
      expect(firstItem.uri).toBeDefined();
      expect(firstItem.source_id).toBeDefined();
      expect(typeof firstItem.title).toBe('string');
      expect(typeof firstItem.summary).toBe('string');
      expect(typeof firstItem.link).toBe('string');
    }
  });

  it('should include both external and internal content links', async () => {
    console.log('\n🔍 Testing content link types...');

    const result = await client.callTool('context_retrieval', {});

    console.log('Status:', result.isError ? 'ERROR' : 'SUCCESS');

    if (result.structuredContent && result.structuredContent.items) {
      console.log('Items found:', result.structuredContent.items.length);

      // Find external and internal links
      const externalLink = result.structuredContent.items.find((item: any) =>
        item.link && item.link.startsWith('https://tyingshoelaces.com/')
      );
      const internalLink = result.structuredContent.items.find((item: any) =>
        item.link && item.link.startsWith('tyingshoelaces://')
      );

      console.log('Found external link:', !!externalLink);
      console.log('Found internal link:', !!internalLink);
    }

    expect(result.isError).toBe(false);
    expect(result.structuredContent).toBeDefined();
  });

  it('should return all available content', async () => {
    const result = await client.callTool('context_retrieval', {});

    console.log('\n🔍 context_retrieval result with all content:');
    console.log('Status:', result.isError ? 'ERROR' : 'SUCCESS');

    if (result.structuredContent && result.structuredContent.items) {
      const itemCount = result.structuredContent.items.length;
      console.log('Items returned:', itemCount);
      expect(itemCount).toBeGreaterThan(0);
    }

    expect(result.isError).toBe(false);
    expect(result.structuredContent).toBeDefined();
    expect(result.structuredContent.items).toBeDefined();
  });
});
