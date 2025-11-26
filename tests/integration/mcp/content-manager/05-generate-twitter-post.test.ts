import { describe, it, expect, beforeAll, afterAll } from 'vitest';
import { generateAdminToken, createContext, McpTestClient } from '@test/utils';
import config from '@test/config';

describe('MCP: content-manager - generate_twitter_post tool', () => {
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

  it('should generate a single Twitter post', async () => {
    const result = await client.callTool('generate_twitter_post', {
      topic: 'The importance of good API design',
      post_type: 'single',
    });

    console.log('\n🐦 Single Twitter Post Result:');
    console.log('Status:', result.isError ? 'ERROR' : 'SUCCESS');

    if (!result.isError && result.structuredContent) {
      const response = result.structuredContent as any;
      console.log('  Platform:', response.platform);
      console.log('  Content:', response.content);
      console.log('  Character Count:', response.character_count);
    }

    expect(result.isError).toBe(false);
    expect(result.structuredContent).toBeDefined();

    const response = result.structuredContent as any;
    expect(response.platform).toBe('twitter');
    expect(response.content).toBeTruthy();
    expect(response.character_count).toBeLessThanOrEqual(280);
  });

  it('should generate a Twitter thread', async () => {
    const result = await client.callTool('generate_twitter_post', {
      topic: 'Why Rust is gaining adoption in systems programming',
      post_type: 'thread',
      thread_length: 5,
    });

    console.log('\n🧵 Twitter Thread Result:');
    console.log('Status:', result.isError ? 'ERROR' : 'SUCCESS');

    expect(result.isError).toBe(false);
    const response = result.structuredContent as any;
    expect(response.platform).toBe('twitter');
    expect(response.content).toBeTruthy();
  });

  it('should generate punchy/contrarian Twitter post', async () => {
    const result = await client.callTool('generate_twitter_post', {
      topic: 'Overengineering in software projects',
      tone: 'contrarian',
    });

    console.log('\n⚡ Contrarian Twitter Post:');
    console.log('Status:', result.isError ? 'ERROR' : 'SUCCESS');

    expect(result.isError).toBe(false);
    const response = result.structuredContent as any;
    expect(response.platform).toBe('twitter');
    expect(response.character_count).toBeLessThanOrEqual(280);
  });

  it('should generate educational Twitter post', async () => {
    const result = await client.callTool('generate_twitter_post', {
      topic: 'Understanding database indexing strategies',
      tone: 'educational',
    });

    console.log('\n📚 Educational Twitter Post:');
    console.log('Status:', result.isError ? 'ERROR' : 'SUCCESS');

    expect(result.isError).toBe(false);
    const response = result.structuredContent as any;
    expect(response.platform).toBe('twitter');
  });

  it('should respect 280 character limit per tweet', async () => {
    const result = await client.callTool('generate_twitter_post', {
      topic: 'Distributed systems complexities and trade-offs',
    });

    expect(result.isError).toBe(false);
    const response = result.structuredContent as any;

    // If it's a thread, check each tweet
    if (response.content.includes('\n---\n')) {
      const tweets = response.content.split('\n---\n');
      tweets.forEach((tweet: string, index: number) => {
        const charCount = tweet.trim().length;
        expect(charCount).toBeLessThanOrEqual(280);
        console.log(`Tweet ${index + 1}: ${charCount} characters`);
      });
    } else {
      expect(response.character_count).toBeLessThanOrEqual(280);
    }
  });

  it('should handle missing required parameters', async () => {
    console.log('\n⚠️  Twitter Post Error Handling (missing topic):');

    try {
      await client.callTool('generate_twitter_post', {
        post_type: 'single',
      });
      expect(false).toBe(true); // Should not reach here
    } catch (error: any) {
      console.log('Expected error caught:', error.message);
      expect(error.message).toContain('Missing required parameter');
    }
  });
});
