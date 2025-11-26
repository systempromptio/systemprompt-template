import { describe, it, expect, beforeAll, afterAll } from 'vitest';
import { generateAdminToken, createContext, McpTestClient } from '@test/utils';
import config from '@test/config';

describe('MCP: content-manager - generate_linkedin_post tool', () => {
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

  it('should generate a professional LinkedIn post', async () => {
    const result = await client.callTool('generate_linkedin_post', {
      topic: 'The future of remote work in tech teams',
      post_type: 'career_lesson',
    });

    console.log('\n💼 LinkedIn Post Generation Result:');
    console.log('Status:', result.isError ? 'ERROR' : 'SUCCESS');

    if (!result.isError && result.structuredContent) {
      const response = result.structuredContent as any;
      console.log('  Platform:', response.platform);
      console.log('  Content length:', response.character_count);
      console.log('  Content preview:', response.content.substring(0, 150) + '...');
    }

    expect(result.isError).toBe(false);
    expect(result.structuredContent).toBeDefined();

    const response = result.structuredContent as any;
    expect(response.platform).toBe('linkedin');
    expect(response.content).toBeTruthy();
    expect(response.character_count).toBeGreaterThan(0);
  }, { timeout: 120000 });

  it('should generate LinkedIn post with data-driven approach', async () => {
    const result = await client.callTool('generate_linkedin_post', {
      topic: 'Scaling infrastructure challenges',
      post_type: 'data_driven',
      keywords: ['infrastructure', 'scaling', 'performance'],
    });

    console.log('\n📊 Data-Driven LinkedIn Post:');
    console.log('Status:', result.isError ? 'ERROR' : 'SUCCESS');

    expect(result.isError).toBe(false);
    const response = result.structuredContent as any;
    expect(response.platform).toBe('linkedin');
    expect(response.content).toBeTruthy();
  }, { timeout: 120000 });

  it('should generate contrarian LinkedIn post', async () => {
    const result = await client.callTool('generate_linkedin_post', {
      topic: 'Why your team might not need microservices',
      post_type: 'contrarian_take',
    });

    console.log('\n⚡ Contrarian LinkedIn Post:');
    console.log('Status:', result.isError ? 'ERROR' : 'SUCCESS');

    expect(result.isError).toBe(false);
    const response = result.structuredContent as any;
    expect(response.platform).toBe('linkedin');
    expect(response.content).toBeTruthy();
  }, { timeout: 120000 });

  it('should generate short-form LinkedIn post', async () => {
    const result = await client.callTool('generate_linkedin_post', {
      topic: 'Quick tips for code reviews',
      post_type: 'provocative_question',
      additional_instructions: 'Keep this post concise and punchy, around 500-800 characters',
    });

    console.log('\n📝 Short-Form LinkedIn Post:');
    console.log('Status:', result.isError ? 'ERROR' : 'SUCCESS');

    expect(result.isError).toBe(false);
    const response = result.structuredContent as any;
    expect(response.character_count).toBeGreaterThan(0);
  }, { timeout: 120000 });

  it('should generate long-form LinkedIn post', async () => {
    const result = await client.callTool('generate_linkedin_post', {
      topic: 'Deep dive: Building scalable systems',
      post_type: 'personal_failure',
      keywords: ['scalability', 'distributed systems', 'architecture'],
      additional_instructions: 'Create a detailed, comprehensive post with personal insights and lessons learned',
    });

    console.log('\n📚 Long-Form LinkedIn Post:');
    console.log('Status:', result.isError ? 'ERROR' : 'SUCCESS');

    expect(result.isError).toBe(false);
    const response = result.structuredContent as any;
    expect(response.character_count).toBeGreaterThan(0);
  }, { timeout: 120000 });

  it('should handle missing required parameters', async () => {
    console.log('\n⚠️  LinkedIn Post Error Handling (missing topic):');

    try {
      await client.callTool('generate_linkedin_post', {
        keywords: ['test'],
      });
      expect(false).toBe(true); // Should not reach here
    } catch (error: any) {
      console.log('Expected error caught:', error.message);
      expect(error.message).toContain('Missing required parameter');
    }
  });
});
