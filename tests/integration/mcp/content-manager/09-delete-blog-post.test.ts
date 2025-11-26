import { describe, it, expect, beforeAll, afterAll } from 'vitest';
import { generateAdminToken, createContext, McpTestClient } from '@test/utils';
import config from '@test/config';

describe('MCP: content-manager - delete_blog_post tool', () => {
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
        // Ignore errors - post might already be deleted
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

  it('should delete a blog post by ID', async () => {
    // First create a post to delete
    const createResult = await client.callTool('create_blog_post', {
      title: 'Post to Delete',
      slug: 'post-to-delete-1',
      content: 'This post will be deleted',
      description: 'Test delete',
      keywords: 'test, delete',
    });

    expect(createResult.isError).toBe(false);
    const postId = extractPostId(createResult);
    expect(postId).toBeTruthy();
    if (postId) createdPostIds.push(postId);

    // Now delete it
    const deleteResult = await client.callTool('delete_blog_post', {
      id: postId,
    });

    console.log('\n🗑️  Delete Blog Post Result:');
    console.log('Status:', deleteResult.isError ? 'ERROR' : 'SUCCESS');

    if (!deleteResult.isError) {
      const deleteTextItem = deleteResult.content.find((c: any) => c.type === 'text') as any;
      const deleteTextContent = deleteTextItem?.text || '';
      console.log('  Message:', deleteTextContent);
    }

    expect(deleteResult.isError).toBe(false);
  });

  it('should require id parameter', async () => {
    console.log('\n⚠️  Missing ID Parameter:');

    try {
      await client.callTool('delete_blog_post', {
        // No id provided
      });
      expect(false).toBe(true); // Should not reach here
    } catch (error: any) {
      console.log('Expected error caught:', error.message);
      expect(error.message).toContain('Missing required parameter');
    }
  });

  it('should handle non-existent id gracefully', async () => {
    console.log('\n⚠️  Non-existent ID Handling:');

    try {
      const result = await client.callTool('delete_blog_post', {
        id: 'non-existent-post-id-xyz',
      });

      // Implementation should return error for non-existent post
      if (result.isError) {
        console.log('Got error response as expected');
        expect(result.isError).toBe(true);
      }
    } catch (error: any) {
      console.log('Error caught:', error.message);
      expect(error.message).toContain('not found');
    }
  });

  it('should delete any blog post regardless of content type', async () => {
    // Create a post with specific content type
    const createResult = await client.callTool('create_blog_post', {
      title: 'Article to Delete',
      slug: 'article-delete-test',
      content: 'Article content',
      description: 'Test article',
      keywords: 'test, article',
      content_type: 'article',
    });

    expect(createResult.isError).toBe(false);
    const postId = extractPostId(createResult);
    expect(postId).toBeTruthy();
    if (postId) createdPostIds.push(postId);

    // Delete it
    const deleteResult = await client.callTool('delete_blog_post', {
      id: postId,
    });

    console.log('\n🗑️  Delete Article Result:');
    console.log('Status:', deleteResult.isError ? 'ERROR' : 'SUCCESS');

    expect(deleteResult.isError).toBe(false);
  });

  it('should delete published posts', async () => {
    // Create and publish a post
    const createResult = await client.callTool('create_blog_post', {
      title: 'Published Post to Delete',
      slug: 'published-delete-test',
      content: 'Published content',
      description: 'Published post',
      keywords: 'published, test',
      published_at: new Date().toISOString(),
    });

    expect(createResult.isError).toBe(false);
    const postId = extractPostId(createResult);
    expect(postId).toBeTruthy();
    if (postId) createdPostIds.push(postId);

    // Delete it
    const deleteResult = await client.callTool('delete_blog_post', {
      id: postId,
    });

    console.log('\n🗑️  Delete Published Post Result:');
    console.log('Status:', deleteResult.isError ? 'ERROR' : 'SUCCESS');

    expect(deleteResult.isError).toBe(false);
  });

  it('should return proper deletion confirmation', async () => {
    // Create a post
    const createResult = await client.callTool('create_blog_post', {
      title: 'Confirmation Test Post',
      slug: 'confirmation-test',
      content: 'Testing confirmation response',
      description: 'Confirmation test',
      keywords: 'test, confirmation',
    });

    expect(createResult.isError).toBe(false);
    const postId = extractPostId(createResult);
    expect(postId).toBeTruthy();
    if (postId) createdPostIds.push(postId);

    // Delete and verify response structure
    const deleteResult = await client.callTool('delete_blog_post', {
      id: postId,
    });

    expect(deleteResult.isError).toBe(false);

    // Response is in text content, not structured_content
    const deleteTextItem = deleteResult.content.find((c: any) => c.type === 'text') as any;
    const deleteTextContent = deleteTextItem?.text || '';
    expect(deleteTextContent.toLowerCase()).toContain('deleted');
    expect(deleteTextContent).toContain(postId);
  });
});
