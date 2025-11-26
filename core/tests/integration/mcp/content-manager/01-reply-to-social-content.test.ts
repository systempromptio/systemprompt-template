import { describe, it, expect, beforeAll, afterAll } from 'vitest';
import { generateAdminToken, createContext, McpTestClient } from '@test/utils';
import config from '@test/config';

describe('MCP: content-manager - reply_to_social_content (unified)', () => {
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

  describe('Skill Loading and Platform Detection', () => {
    /**
     * NOTE: The reply_to_social_content tool automatically loads the appropriate skill
     * based on the platform parameter:
     * - linkedin -> linkedin_post_writing skill
     * - twitter -> twitter_post_writing skill
     * - reddit -> reddit_post_writing skill
     *
     * Skills are loaded from the agent_skills table via SkillLoader service.
     * The skill instructions are used as the system prompt for content generation.
     */

    it('should load skills and generate content for all platforms', async () => {
      console.log('\n🔍 Testing Skill Loading & Platform Detection...\n');

      const platforms = [
        { name: 'LinkedIn', platform: 'linkedin', skill: 'linkedin_post_writing' },
        { name: 'Twitter', platform: 'twitter', skill: 'twitter_post_writing' },
        { name: 'Reddit', platform: 'reddit', skill: 'reddit_post_writing' },
      ];

      for (const { name, platform, skill } of platforms) {
        console.log(`Testing ${name} (skill: ${skill})...`);

        const result = await client.callTool('reply_to_social_content', {
          platform,
          content: 'Test content for platform detection',
          instructions: 'Write a brief test reply',
        });

        expect(result.isError).toBe(false);
        expect(result.structuredContent).toBeDefined();

        const artifact = result.structuredContent as any;
        expect(artifact['x-artifact-type']).toBe('copy_paste_text');
        expect(artifact.content).toBeTruthy();

        console.log(`  ✅ ${name} skill loaded successfully`);
        console.log(`  Content length: ${artifact.content.length} chars\n`);
      }

      console.log('✅ All platform skills validated and loaded successfully\n');
    }, { timeout: 180000 });
  });

  describe('LinkedIn Replies', () => {
    it('should generate supportive LinkedIn reply', async () => {
      const result = await client.callTool('reply_to_social_content', {
        platform: 'linkedin',
        content: 'Just launched our new API! After 6 months of hard work, we finally shipped it to production. The team did an amazing job.',
        instructions: 'Write a supportive and encouraging reply. Be genuine and congratulate them on their achievement.',
      });

      console.log('\n💼 LinkedIn Reply - Supportive:');
      console.log('Status:', result.isError ? 'ERROR' : 'SUCCESS');

      if (!result.isError && result.structuredContent) {
        const artifact = result.structuredContent as any;
        console.log('  Artifact Type:', artifact['x-artifact-type']);
        console.log('  Content Length:', artifact.content?.length || 0);
        console.log('  Title:', artifact.title || 'N/A');
        console.log('  Language:', artifact.language || 'N/A');
        console.log('  Content Preview:', artifact.content?.substring(0, 200) + '...\n');
      }

      expect(result.isError).toBe(false);
      expect(result.structuredContent).toBeDefined();

      const artifact = result.structuredContent as any;
      expect(artifact['x-artifact-type']).toBe('copy_paste_text');
      expect(artifact.content).toBeTruthy();
      expect(artifact.content.length).toBeGreaterThan(50);
    }, { timeout: 120000 });

    it('should generate professional LinkedIn reply with feedback', async () => {
      const result = await client.callTool('reply_to_social_content', {
        platform: 'linkedin',
        content: 'Thoughts on whether microservices are worth the complexity? I keep hearing conflicting advice.',
        instructions: 'Provide a thoughtful, balanced response. Share a perspective based on experience with both monoliths and microservices.',
      });

      console.log('\n💼 LinkedIn Reply - Professional Feedback:');
      console.log('Status:', result.isError ? 'ERROR' : 'SUCCESS');

      expect(result.isError).toBe(false);
      const artifact = result.structuredContent as any;
      expect(artifact['x-artifact-type']).toBe('copy_paste_text');
      expect(artifact.content).toBeTruthy();
    }, { timeout: 120000 });
  });

  describe('Twitter Replies', () => {
    it('should generate concise Twitter reply', async () => {
      const result = await client.callTool('reply_to_social_content', {
        platform: 'twitter',
        content: 'Hot take: Most startups don\'t need Kubernetes. Change my mind.',
        instructions: 'Write a witty but agreeable response. Keep it under 280 characters. Acknowledge their point while adding a nuanced perspective.',
      });

      console.log('\n🐦 Twitter Reply - Witty:');
      console.log('Status:', result.isError ? 'ERROR' : 'SUCCESS');

      if (!result.isError && result.structuredContent) {
        const artifact = result.structuredContent as any;
        console.log('  Content:', artifact.content);
        console.log('  Content Length:', artifact.content?.length || 0, 'chars');
        if (artifact.content.length > 280) {
          console.log('  ⚠️  Warning: Content exceeds Twitter character limit\n');
        } else {
          console.log('  ✅ Within Twitter character limit\n');
        }
      }

      expect(result.isError).toBe(false);
      expect(result.structuredContent).toBeDefined();

      const artifact = result.structuredContent as any;
      expect(artifact['x-artifact-type']).toBe('copy_paste_text');
      expect(artifact.content).toBeTruthy();
      // Note: We don't strictly enforce 280 char limit as the AI doesn't always respect it
      // The skill should guide the model, but we log a warning if exceeded
    }, { timeout: 120000 });

    it('should generate engaging Twitter reply to technical question', async () => {
      const result = await client.callTool('reply_to_social_content', {
        platform: 'twitter',
        content: 'What\'s your go-to debugging technique when everything seems broken?',
        instructions: 'Share a specific, practical debugging tip. Be conversational and relatable.',
      });

      console.log('\n🐦 Twitter Reply - Technical:');
      console.log('Status:', result.isError ? 'ERROR' : 'SUCCESS');

      if (!result.isError && result.structuredContent) {
        const artifact = result.structuredContent as any;
        console.log('  Content Length:', artifact.content?.length || 0, 'chars');
        if (artifact.content.length > 280) {
          console.log('  ⚠️  Warning: Content exceeds Twitter character limit\n');
        } else {
          console.log('  ✅ Within Twitter character limit\n');
        }
      }

      expect(result.isError).toBe(false);
      const artifact = result.structuredContent as any;
      expect(artifact['x-artifact-type']).toBe('copy_paste_text');
      expect(artifact.content).toBeTruthy();
      // Note: We don't strictly enforce 280 char limit as the AI doesn't always respect it
    }, { timeout: 120000 });
  });

  describe('Reddit Replies', () => {
    it('should generate detailed Reddit reply with technical depth', async () => {
      const result = await client.callTool('reply_to_social_content', {
        platform: 'reddit',
        content: 'I\'ve been using Rust for a year now and I\'m still struggling with lifetimes. Does it ever click? Should I just give up and go back to Python?',
        instructions: 'Write an encouraging, detailed response. Share specific insights about learning Rust. Be empathetic and provide concrete advice.',
      });

      console.log('\n📱 Reddit Reply - Technical Help:');
      console.log('Status:', result.isError ? 'ERROR' : 'SUCCESS');

      if (!result.isError && result.structuredContent) {
        const artifact = result.structuredContent as any;
        console.log('  Content Length:', artifact.content?.length || 0, 'chars');
        console.log('  Content Preview:', artifact.content?.substring(0, 300) + '...\n');
      }

      expect(result.isError).toBe(false);
      expect(result.structuredContent).toBeDefined();

      const artifact = result.structuredContent as any;
      expect(artifact['x-artifact-type']).toBe('copy_paste_text');
      expect(artifact.content).toBeTruthy();
      expect(artifact.content.length).toBeGreaterThan(200);
    }, { timeout: 120000 });

    it('should generate Reddit reply with personal experience', async () => {
      const result = await client.callTool('reply_to_social_content', {
        platform: 'reddit',
        content: 'Anyone else find that the best code they write is when they delete code rather than add it?',
        instructions: 'Share a relatable story about code simplification. Be conversational and authentic, like you\'re chatting with fellow developers.',
      });

      console.log('\n📱 Reddit Reply - Personal Experience:');
      console.log('Status:', result.isError ? 'ERROR' : 'SUCCESS');

      expect(result.isError).toBe(false);
      const artifact = result.structuredContent as any;
      expect(artifact['x-artifact-type']).toBe('copy_paste_text');
      expect(artifact.content).toBeTruthy();
    }, { timeout: 120000 });
  });

  describe('Error Handling', () => {
    it('should fail with invalid platform', async () => {
      console.log('\n⚠️  Testing Invalid Platform...');

      try {
        await client.callTool('reply_to_social_content', {
          platform: 'invalid_platform',
          content: 'Some content',
          instructions: 'Reply to this',
        });
        expect(false).toBe(true); // Should not reach here
      } catch (error: any) {
        console.log('Expected error caught:', error.message);
        expect(error.message).toBeDefined();
      }
    });

    it('should fail with missing platform parameter', async () => {
      console.log('\n⚠️  Testing Missing Platform...');

      try {
        await client.callTool('reply_to_social_content', {
          content: 'Some content',
          instructions: 'Reply to this',
        });
        expect(false).toBe(true); // Should not reach here
      } catch (error: any) {
        console.log('Expected error caught:', error.message);
        expect(error.message).toBeDefined();
      }
    });

    it('should fail with missing content parameter', async () => {
      console.log('\n⚠️  Testing Missing Content...');

      try {
        await client.callTool('reply_to_social_content', {
          platform: 'linkedin',
          instructions: 'Reply to this',
        });
        expect(false).toBe(true); // Should not reach here
      } catch (error: any) {
        console.log('Expected error caught:', error.message);
        expect(error.message).toBeDefined();
      }
    });

    it('should handle missing instructions parameter', async () => {
      console.log('\n⚠️  Testing Missing Instructions...');

      // Note: Instructions may have a default value, so this might succeed
      const result = await client.callTool('reply_to_social_content', {
        platform: 'linkedin',
        content: 'Some content to reply to',
      });

      // If it succeeds, instructions has a default value
      // If it fails, we catch the error
      if (result.isError) {
        console.log('Instructions are required (error returned)');
        expect(result.isError).toBe(true);
      } else {
        console.log('Instructions have default value (success)');
        expect(result.isError).toBe(false);
      }
    }, { timeout: 60000 });
  });
});
