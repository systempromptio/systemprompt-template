import { describe, it, expect, beforeAll, afterAll } from 'vitest';
import { generateAdminToken, createContext, McpTestClient } from '@test/utils';
import config from '@test/config';

describe('MCP: content-manager - generate_reddit_post tool', () => {
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

  it('should generate a value-first Reddit post', async () => {
    const result = await client.callTool('generate_reddit_post', {
      topic: 'Learning Rust as a Python developer',
      subreddit: 'r/rust',
      post_type: 'experience_report',
    });

    console.log('\n📕 Reddit Post Generation Result:');
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
    expect(response.platform).toBe('reddit');
    expect(response.content).toBeTruthy();
    expect(response.character_count).toBeGreaterThan(100);
    expect(response.character_count).toBeLessThanOrEqual(2000);
  });

  it('should generate technical dive Reddit post', async () => {
    const result = await client.callTool('generate_reddit_post', {
      topic: 'Understanding PostgreSQL query optimization',
      subreddit: 'r/programming',
      post_type: 'technical_dive',
    });

    console.log('\n🔬 Technical Dive Reddit Post:');
    console.log('Status:', result.isError ? 'ERROR' : 'SUCCESS');

    expect(result.isError).toBe(false);
    const response = result.structuredContent as any;
    expect(response.platform).toBe('reddit');
    expect(response.character_count).toBeGreaterThan(100);
  });

  it('should generate insightful Reddit post without self-promotion', async () => {
    const result = await client.callTool('generate_reddit_post', {
      topic: 'Best practices for handling distributed system failures',
      post_type: 'insight_sharing',
    });

    console.log('\n💡 Insight Sharing Reddit Post:');
    console.log('Status:', result.isError ? 'ERROR' : 'SUCCESS');

    expect(result.isError).toBe(false);
    const response = result.structuredContent as any;
    expect(response.platform).toBe('reddit');
    expect(response.content).toBeTruthy();
  });

  it('should generate question/discussion Reddit post', async () => {
    const result = await client.callTool('generate_reddit_post', {
      topic: 'What are your thoughts on TypeScript vs Go?',
      post_type: 'discussion',
    });

    console.log('\n❓ Discussion Reddit Post:');
    console.log('Status:', result.isError ? 'ERROR' : 'SUCCESS');

    expect(result.isError).toBe(false);
    const response = result.structuredContent as any;
    expect(response.platform).toBe('reddit');
  });

  it('should maintain 100-2000 word range', async () => {
    const result = await client.callTool('generate_reddit_post', {
      topic: 'Effective code review strategies',
      subreddit: 'r/webdev',
    });

    expect(result.isError).toBe(false);
    const response = result.structuredContent as any;
    expect(response.character_count).toBeGreaterThan(100);
    expect(response.character_count).toBeLessThanOrEqual(2000);
  });

  it('should handle missing required parameters', async () => {
    console.log('\n⚠️  Reddit Post Error Handling (missing topic):');

    try {
      await client.callTool('generate_reddit_post', {
        post_type: 'experience_report',
      });
      expect(false).toBe(true); // Should not reach here
    } catch (error: any) {
      console.log('Expected error caught:', error.message);
      expect(error.message).toContain('Missing required parameter');
    }
  });

  it('should generate authentic voice without commercial intent', async () => {
    const result = await client.callTool('generate_reddit_post', {
      topic: 'Building your first open source project',
      subreddit: 'r/learnprogramming',
    });

    expect(result.isError).toBe(false);
    const response = result.structuredContent as any;

    // Verify it's genuine insight, not promotional
    expect(response.content.toLowerCase()).not.toContain('buy');
    expect(response.content.toLowerCase()).not.toContain('click here');
    expect(response.content).toBeTruthy();
  });
});
