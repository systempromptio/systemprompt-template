import { describe, it, expect, beforeAll, afterAll } from 'vitest';
import { generateAdminToken, createContext, McpTestClient } from '@test/utils';
import config from '@test/config';

describe('MCP: content-manager - update_blog_post tool', () => {
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

  it('should update blog post content', async () => {
    // First create a post to update
    const createResult = await client.callTool('create_blog_post', {
      title: 'Original Title',
      slug: 'post-to-update',
      content: 'Original content here',
      description: 'Original description',
      keywords: 'original, test',
    });

    expect(createResult.isError).toBe(false);

    const postId = extractPostId(createResult);
    expect(postId).toBeTruthy();
    if (postId) createdPostIds.push(postId);

    // Now update it
    const updateResult = await client.callTool('update_blog_post', {
      id: postId,
      title: 'Updated Title',
      content: 'Updated content with improvements',
    });

    console.log('\n✏️  Update Blog Post Result:');
    console.log('Status:', updateResult.isError ? 'ERROR' : 'SUCCESS');

    if (!updateResult.isError && updateResult.structuredContent) {
      const response = updateResult.structuredContent as any;
      console.log('  Title:', response.title);
    }

    expect(updateResult.isError).toBe(false);
    const response = updateResult.structuredContent as any;
    expect(response.title).toBe('Updated Title');
  });

  it('should update blog post publish date', async () => {
    // Create a post
    const createResult = await client.callTool('create_blog_post', {
      title: 'Post to Reschedule',
      slug: 'post-to-reschedule',
      content: 'Content here',
      description: 'Description',
      keywords: 'test, update',
    });

    expect(createResult.isError).toBe(false);
    const postId = extractPostId(createResult);
    expect(postId).toBeTruthy();
    if (postId) createdPostIds.push(postId);

    // Update publish date
    const newPublishDate = new Date(Date.now() + 86400000).toISOString(); // tomorrow
    const updateResult = await client.callTool('update_blog_post', {
      id: postId,
      published_at: newPublishDate,
    });

    console.log('\n📢 Update Publish Date Result:');
    console.log('Status:', updateResult.isError ? 'ERROR' : 'SUCCESS');

    expect(updateResult.isError).toBe(false);
  });

  it('should update blog post description', async () => {
    const createResult = await client.callTool('create_blog_post', {
      title: 'Post for Description Update',
      slug: 'description-update-test',
      content: 'Full content here',
      description: 'Old description',
      keywords: 'test, update',
    });

    expect(createResult.isError).toBe(false);
    const postId = extractPostId(createResult);
    expect(postId).toBeTruthy();
    if (postId) createdPostIds.push(postId);

    const updateResult = await client.callTool('update_blog_post', {
      id: postId,
      description: 'New and improved description that better describes the content',
    });

    console.log('\n📝 Update Description Result:');
    console.log('Status:', updateResult.isError ? 'ERROR' : 'SUCCESS');

    expect(updateResult.isError).toBe(false);
    const response = updateResult.structuredContent as any;
    expect(response.excerpt).toBe('New and improved description that better describes the content');
  });

  it('should update blog post tags and category', async () => {
    const createResult = await client.callTool('create_blog_post', {
      title: 'Tag Update Test',
      slug: 'tag-update-test',
      content: 'Content here',
      description: 'Test description',
      keywords: 'test, tags',
      tags: ['old-tag'],
      category_id: 'old-category',
    });

    expect(createResult.isError).toBe(false);
    const postId = extractPostId(createResult);
    expect(postId).toBeTruthy();
    if (postId) createdPostIds.push(postId);

    const updateResult = await client.callTool('update_blog_post', {
      id: postId,
      tags: ['new-tag', 'relevant-tag'],
      category_id: 'new-category',
    });

    console.log('\n🏷️  Update Tags Result:');
    console.log('Status:', updateResult.isError ? 'ERROR' : 'SUCCESS');

    expect(updateResult.isError).toBe(false);
  });

  it('should handle missing id', async () => {
    console.log('\n⚠️  Missing ID Error:');

    try {
      await client.callTool('update_blog_post', {
        title: 'Update without ID',
        content: 'This should fail',
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
      const result = await client.callTool('update_blog_post', {
        id: 'non-existent-id-12345',
        title: 'Update non-existent',
        content: 'This post does not exist',
      });

      // Should either return error or result
      if (result.isError) {
        console.log('Got error as expected');
        expect(result.isError).toBe(true);
      }
    } catch (error: any) {
      console.log('Error caught:', error.message);
      expect(error.message).toContain('not found') || expect(error.message).toContain('not exist');
    }
  });
});
