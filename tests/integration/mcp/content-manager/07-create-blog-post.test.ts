import { describe, it, expect, beforeAll, afterAll } from 'vitest';
import { generateAdminToken, createContext, McpTestClient } from '@test/utils';
import config from '@test/config';

describe('MCP: content-manager - create_blog_post tool', () => {
  let client: McpTestClient;
  let adminToken: string;
  let contextId: string;
  const createdPostIds: string[] = [];

  function extractPostId(result: any): string | null {
    const textContent = result.content.find((c: any) => c.type === 'text')?.text || '';
    const idMatch = textContent.match(/ID:\s*([a-f0-9-]+)/);
    return idMatch ? idMatch[1] : null;
  }

  async function cleanupTestPosts() {
    for (const postId of createdPostIds) {
      try {
        await client.callTool('delete_blog_post', { id: postId });
      } catch (error) {
        // Ignore errors - post might not exist
      }
    }
    createdPostIds.length = 0;
  }

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
    await cleanupTestPosts();
    await client.close();
  });

  it('should create a new blog post', async () => {
    const result = await client.callTool('create_blog_post', {
      title: 'Understanding Distributed Systems: A Practical Guide',
      slug: 'distributed-systems-guide',
      content: '# Understanding Distributed Systems\n\nDistributed systems are complex but essential...',
      description: 'A practical guide to building reliable distributed systems',
      keywords: 'distributed systems, architecture, scalability',
      published_at: new Date().toISOString(),
    });

    const postId = extractPostId(result);
    if (postId) createdPostIds.push(postId);

    console.log('\n📝 Create Blog Post Result:');
    console.log('Status:', result.isError ? 'ERROR' : 'SUCCESS');

    if (!result.isError && result.structuredContent) {
      const response = result.structuredContent as any;
      console.log('  Title:', response.title);
      console.log('  Slug:', response.slug);
    }

    expect(result.isError).toBe(false);
    expect(result.structuredContent).toBeDefined();

    const response = result.structuredContent as any;
    expect(response.title).toBe('Understanding Distributed Systems: A Practical Guide');
    expect(response.slug).toBe('distributed-systems-guide');
    expect(response['x-artifact-type']).toBe('blog');
  });

  it('should create blog post with tags and category', async () => {
    const result = await client.callTool('create_blog_post', {
      title: 'Scaling Your Backend Infrastructure',
      slug: 'scaling-backend',
      content: '# Scaling Your Backend\n\nAs your application grows...',
      description: 'Best practices for scaling backend systems',
      keywords: 'scaling, infrastructure, devops',
      tags: ['infrastructure', 'scaling', 'devops'],
      category_id: 'engineering',
    });

    const postId = extractPostId(result);
    if (postId) createdPostIds.push(postId);

    console.log('\n🏷️  Blog Post with Tags Result:');
    console.log('Status:', result.isError ? 'ERROR' : 'SUCCESS');

    expect(result.isError).toBe(false);
    const response = result.structuredContent as any;
    expect(response.title).toBe('Scaling Your Backend Infrastructure');
  });

  it('should create blog post with content type', async () => {
    const result = await client.callTool('create_blog_post', {
      title: 'Work in Progress: Advanced Rust Patterns',
      slug: 'rust-patterns-wip',
      content: '# Rust Patterns\n\nExploring advanced patterns...',
      description: 'An exploration of advanced Rust programming patterns',
      keywords: 'rust, patterns, programming',
      content_type: 'article',
    });

    const postId = extractPostId(result);
    if (postId) createdPostIds.push(postId);

    console.log('\n📋 Article Content Type Result:');
    console.log('Status:', result.isError ? 'ERROR' : 'SUCCESS');

    expect(result.isError).toBe(false);
    const response = result.structuredContent as any;
    expect(response.title).toBe('Work in Progress: Advanced Rust Patterns');
  });

  it('should create blog post with featured image', async () => {
    const result = await client.callTool('create_blog_post', {
      title: 'The Future of Web Performance',
      slug: 'web-performance-future',
      content: '# Web Performance\n\nOptimization strategies...',
      description: 'Exploring the future of web performance optimization',
      keywords: 'web performance, optimization, speed',
      image: 'https://example.com/images/performance.png',
    });

    const postId = extractPostId(result);
    if (postId) createdPostIds.push(postId);

    console.log('\n🖼️  Blog Post with Featured Image:');
    console.log('Status:', result.isError ? 'ERROR' : 'SUCCESS');

    expect(result.isError).toBe(false);
    const response = result.structuredContent as any;
    expect(response.title).toBe('The Future of Web Performance');
  });

  it('should handle missing required parameters', async () => {
    console.log('\n⚠️  Blog Post Error Handling (missing title):');

    try {
      await client.callTool('create_blog_post', {
        slug: 'missing-title',
        content: 'This post is missing a title',
      });
      expect(false).toBe(true); // Should not reach here
    } catch (error: any) {
      console.log('Expected error caught:', error.message);
      expect(error.message).toContain('Missing required parameter');
    }
  });

  it('should handle duplicate slug', async () => {
    console.log('\n⚠️  Duplicate Slug Handling:');

    // First, create a post
    const firstResult = await client.callTool('create_blog_post', {
      title: 'Original Post',
      slug: 'unique-post-slug-test',
      content: 'Original content',
      description: 'Original description',
      keywords: 'original, test',
    });

    const postId = extractPostId(firstResult);
    if (postId) createdPostIds.push(postId);

    expect(firstResult.isError).toBe(false);

    // Try to create another with same slug
    try {
      await client.callTool('create_blog_post', {
        title: 'Duplicate Post',
        slug: 'unique-post-slug-test',
        content: 'Duplicate content',
        description: 'Duplicate description',
        keywords: 'duplicate, test',
      });
      // Should fail with duplicate slug error
      expect(false).toBe(true); // Should not reach here
    } catch (error: any) {
      console.log('Duplicate slug error (expected):', error.message);
      expect(error.message).toContain('slug');
    }
  });
});
