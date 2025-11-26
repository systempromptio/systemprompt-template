import { describe, it, expect, beforeAll, afterAll } from 'vitest';
import { generateAdminToken, createContext, McpTestClient } from '@test/utils';
import config from '@test/config';
import { DatabaseClient } from '@test/database';

describe('MCP: content-manager - generate_social_content with parent linking', () => {
  let client: McpTestClient;
  let adminToken: string;
  let contextId: string;
  let db: DatabaseClient;
  let parentBlogId: string;

  beforeAll(async () => {
    adminToken = generateAdminToken();
    contextId = await createContext(adminToken, config.apiBaseUrl);
    db = new DatabaseClient();

    client = new McpTestClient({
      baseUrl: config.apiBaseUrl,
      serverName: 'content-manager',
      token: adminToken,
      contextId,
    });
    await client.connect();

    // Create a parent blog post
    const blogResult = await client.callTool('create_blog_post', {
      title: 'Test Blog Post for Social Generation',
      slug: `test-social-parent-${Date.now()}`,
      content: '# Test Content\n\nThis is a test blog post that will be used to generate social content.',
      description: 'Test blog for social content generation',
      keywords: 'test,social,parent',
      content_type: 'article',
      tags: ['test', 'social'],
    });

    expect(blogResult.isError).toBe(false);

    // Extract blog ID from the response
    const blogResponse = blogResult.structuredContent as any;
    parentBlogId = blogResponse.content_id;
    console.log('Created parent blog post with ID:', parentBlogId);
  });

  afterAll(async () => {
    // Cleanup: delete the parent blog (should cascade delete social content)
    if (parentBlogId) {
      await client.callTool('delete_blog_post', { content_id: parentBlogId });
    }
    await client.close();
    await db.close();
  });

  it('should generate LinkedIn social content linked to parent blog', async () => {
    const result = await client.callTool('generate_social_content', {
      platform: 'linkedin',
      content_id: parentBlogId,
      instructions: 'Create a professional post highlighting the key points',
    });

    console.log('\n💼 LinkedIn Social Content Generation:');
    console.log('Status:', result.isError ? 'ERROR' : 'SUCCESS');
    console.log('Content:', JSON.stringify(result.content, null, 2));

    expect(result.isError).toBe(false);
    expect(result.structuredContent).toBeDefined();

    // Verify the social content was saved to database
    const socialContent = await db.query(
      'SELECT * FROM markdown_content WHERE parent_content_id = $1 AND content_type = $2',
      [parentBlogId, 'social_linkedin']
    );

    expect(socialContent.rows.length).toBeGreaterThan(0);
    const savedPost = socialContent.rows[0];

    console.log('Saved social content ID:', savedPost.id);
    console.log('Parent content ID:', savedPost.parent_content_id);
    console.log('Content type:', savedPost.content_type);

    expect(savedPost.parent_content_id).toBe(parentBlogId);
    expect(savedPost.content_type).toBe('social_linkedin');
    expect(savedPost.content).toBeTruthy();

    // Verify tags were inherited
    const tags = await db.query(
      'SELECT t.name FROM markdown_tags t INNER JOIN markdown_content_tags ct ON t.id = ct.tag_id WHERE ct.content_id = $1',
      [savedPost.id]
    );

    console.log('Inherited tags:', tags.rows.map(r => r.name));
    expect(tags.rows.length).toBeGreaterThan(0);
  }, { timeout: 120000 });

  it('should generate Twitter social content linked to parent blog', async () => {
    const result = await client.callTool('generate_social_content', {
      platform: 'twitter',
      content_id: parentBlogId,
      instructions: 'Create a concise thread about the main points',
    });

    console.log('\n🐦 Twitter Social Content Generation:');
    console.log('Status:', result.isError ? 'ERROR' : 'SUCCESS');

    expect(result.isError).toBe(false);

    // Verify the social content was saved
    const socialContent = await db.query(
      'SELECT * FROM markdown_content WHERE parent_content_id = $1 AND content_type = $2',
      [parentBlogId, 'social_twitter']
    );

    expect(socialContent.rows.length).toBeGreaterThan(0);
    expect(socialContent.rows[0].content_type).toBe('social_twitter');
  }, { timeout: 120000 });

  it('should delete social content when parent blog is deleted (cascade)', async () => {
    // First, generate some social content
    await client.callTool('generate_social_content', {
      platform: 'reddit',
      content_id: parentBlogId,
      instructions: 'Create a discussion post for r/programming',
    });

    // Verify social content exists
    const beforeDelete = await db.query(
      'SELECT COUNT(*) as count FROM markdown_content WHERE parent_content_id = $1',
      [parentBlogId]
    );

    const countBefore = parseInt(beforeDelete.rows[0].count);
    console.log('\n🗑️ Social content count before delete:', countBefore);
    expect(countBefore).toBeGreaterThan(0);

    // Delete the parent blog
    const deleteResult = await client.callTool('delete_blog_post', { content_id: parentBlogId });
    expect(deleteResult.isError).toBe(false);

    // Verify social content was cascade deleted
    const afterDelete = await db.query(
      'SELECT COUNT(*) as count FROM markdown_content WHERE parent_content_id = $1',
      [parentBlogId]
    );

    const countAfter = parseInt(afterDelete.rows[0].count);
    console.log('Social content count after delete:', countAfter);
    expect(countAfter).toBe(0);

    // Clear parentBlogId so afterAll doesn't try to delete again
    parentBlogId = '';
  }, { timeout: 120000 });
});
