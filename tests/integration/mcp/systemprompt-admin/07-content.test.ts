import { describe, it, expect, beforeAll, afterAll } from 'vitest';
import { generateAdminToken, createContext, McpTestClient } from '@test/utils';
import config from '@test/config';

describe('MCP: systemprompt-admin - content tool', () => {
  let client: McpTestClient;
  let adminToken: string;
  let contextId: string;

  beforeAll(async () => {
    adminToken = generateAdminToken();
    contextId = await createContext(adminToken, config.apiBaseUrl);

    client = new McpTestClient({
      baseUrl: config.apiBaseUrl,
      serverName: 'systemprompt-admin',
      token: adminToken,
      contextId,
    });
    await client.connect();
  });

  afterAll(async () => {
    await client.close();
  });

  it('should list tools and find content', async () => {
    const result = await client.listTools();

    console.log('📋 Available admin tools:', JSON.stringify(result.tools.map((t: any) => t.name), null, 2));

    expect(result.tools).toBeDefined();
    const contentTool = result.tools.find((t: any) => t.name === 'content');
    expect(contentTool).toBeDefined();
    expect(contentTool.name).toBe('content');
    expect(contentTool.title).toContain('Content');
  });

  it('should return content data with loaded fixtures', async () => {
    const result = await client.callTool('content', { time_range: '30d' });

    console.log('\n📄 Content Tool with Fixtures:');
    console.log('Status:', result.isError ? 'ERROR' : 'SUCCESS');

    expect(result.isError).toBe(false);
    expect(result.structuredContent).toBeDefined();

    const content = result.structuredContent as any;
    expect(content.sections).toBeDefined();
    expect(content.sections.length).toBeGreaterThan(0);

    // Verify top content section has data
    const topContentSection = content.sections?.find((s: any) => s.section_id === 'top_content');
    expect(topContentSection).toBeDefined();
    expect(topContentSection.data).toBeDefined();
    expect(topContentSection.data.items).toBeDefined();

    console.log('✅ Top content items:', topContentSection?.data?.items?.length || 0);
    if (topContentSection?.data?.items?.length > 0) {
      console.log('Sample item:', JSON.stringify(topContentSection.data.items[0], null, 2));
    }

    // Verify content trends section has data
    const trendsSection = content.sections?.find((s: any) => s.section_id === 'trends');
    expect(trendsSection).toBeDefined();
    expect(trendsSection.data).toBeDefined();
    expect(trendsSection.data.items).toBeDefined();

    console.log('✅ Trend items:', trendsSection?.data?.items?.length || 0);
  });

  it('should execute content with default (30d, views) parameters', async () => {
    const input = {};
    console.log('\n📄 Content Tool Input:', JSON.stringify(input, null, 2));

    const result = await client.callTool('content', input);

    console.log('\n📄 Content result (default 30d, views):');
    console.log('Status:', result.isError ? 'ERROR' : 'SUCCESS');

    if (result.isError) {
      console.log('Error:', JSON.stringify(result, null, 2));
    } else {
      console.log('Content type:', result.content[0]?.type);

      if (result.structuredContent) {
        const content = result.structuredContent as any;
        console.log('\n📊 Content Dashboard:');
        console.log('  Title:', content.title);
        console.log('  Description:', content.description);
        console.log('  Sections:', content.sections?.length || 0);

        if (content.sections) {
          content.sections.forEach((section: any) => {
            console.log(`\n  📍 Section: ${section.title}`);
            console.log(`     ID: ${section.section_id}`);
            console.log(`     Type: ${section.section_type}`);
            console.log(`     Data keys: ${Object.keys(section.data || {}).join(', ')}`);
          });
        }
      }
    }

    expect(result.isError).toBe(false);
    expect(result.content).toBeDefined();
    expect(result.structuredContent).toBeDefined();

    const content = result.structuredContent as any;
    expect(content.title).toBe('Content Performance Analytics');
    expect(content.sections).toBeDefined();
    expect(Array.isArray(content.sections)).toBe(true);
  });

  it('should execute content with 7d time range', async () => {
    const input = { time_range: '7d' };
    console.log('\n📄 Content Tool Input:', JSON.stringify(input, null, 2));

    const result = await client.callTool('content', input);

    console.log('\n📄 Content result (7 days):');
    console.log('Status:', result.isError ? 'ERROR' : 'SUCCESS');

    if (result.structuredContent) {
      const content = result.structuredContent as any;
      console.log('  Description:', content.description);
      expect(content.description).toContain('7 days');
    }

    expect(result.isError).toBe(false);
  });

  it('should execute content with 30d time range', async () => {
    const input = { time_range: '30d' };
    const result = await client.callTool('content', input);

    console.log('\n📄 Content result (30 days):');
    console.log('Status:', result.isError ? 'ERROR' : 'SUCCESS');

    if (result.structuredContent) {
      const content = result.structuredContent as any;
      console.log('  Description:', content.description);
      expect(content.description).toContain('30 days');
    }

    expect(result.isError).toBe(false);
  });

  it('should execute content with 90d time range', async () => {
    const input = { time_range: '90d' };
    const result = await client.callTool('content', input);

    console.log('\n📄 Content result (90 days):');
    console.log('Status:', result.isError ? 'ERROR' : 'SUCCESS');

    if (result.structuredContent) {
      const content = result.structuredContent as any;
      console.log('  Description:', content.description);
      expect(content.description).toContain('90 days');
    }

    expect(result.isError).toBe(false);
  });

  it('should have expected content sections', async () => {
    const result = await client.callTool('content', {});

    expect(result.isError).toBe(false);

    const content = result.structuredContent as any;
    expect(content.sections).toBeDefined();

    // Check for expected section types (may or may not have data)
    const sectionIds = content.sections.map((s: any) => s.section_id);
    console.log('\n📊 Available sections:', sectionIds);

    // Content sections should include at least one of these if data exists
    const possibleSections = ['top_content', 'trends'];
    const hasValidSection = sectionIds.some((id: string) => possibleSections.includes(id));

    // If we have sections, at least one should be valid
    if (content.sections.length > 0) {
      expect(hasValidSection).toBe(true);
    }
  });

  it('should show top content section if available', async () => {
    const result = await client.callTool('content', { time_range: '30d' });

    expect(result.isError).toBe(false);

    const content = result.structuredContent as any;
    const topContentSection = content.sections?.find((s: any) => s.section_id === 'top_content');

    if (topContentSection) {
      console.log('\n📊 Top Content Section:');
      console.log('  Title:', topContentSection.title);
      console.log('  Type:', topContentSection.section_type);
      console.log('  Items count:', topContentSection.data?.items?.length || 0);

      if (topContentSection.data?.items && topContentSection.data.items.length > 0) {
        console.log('\n  Sample items:');
        topContentSection.data.items.slice(0, 3).forEach((item: any) => {
          console.log(`    - ${item.label}`);
          console.log(`      Views: ${item.value}`);
          console.log(`      Visitors: ${item.badge}`);
          console.log(`      Preview: ${item.secondary}`);
          if (item.link) console.log(`      Link: ${item.link}`);
        });

        expect(topContentSection.section_type).toBe('list');
        expect(topContentSection.data.items).toBeDefined();
        expect(Array.isArray(topContentSection.data.items)).toBe(true);
      } else {
        console.log('  ⚠️  No content items available (expected if no content views exist)');
      }
    } else {
      console.log('⚠️  No top content section (expected if no content data)');
    }
  });

  it('should show content trends if available', async () => {
    const result = await client.callTool('content', { time_range: '30d' });

    expect(result.isError).toBe(false);

    const content = result.structuredContent as any;
    const trendsSection = content.sections?.find((s: any) => s.section_id === 'trends');

    if (trendsSection) {
      console.log('\n📊 Content Trends:');
      console.log('  Title:', trendsSection.title);
      console.log('  Trend items:', trendsSection.data?.items?.length || 0);

      if (trendsSection.data?.items && trendsSection.data.items.length > 0) {
        console.log('\n  Recent trends:');
        trendsSection.data.items.slice(0, 5).forEach((item: any) => {
          console.log(`    - ${item.label}: ${item.value}`);
          console.log(`      Age: ${item.badge}`);
          console.log(`      Preview: ${item.secondary}`);
          if (item.link) console.log(`      Link: ${item.link}`);
        });

        expect(trendsSection.section_type).toBe('list');
        expect(trendsSection.data.items).toBeDefined();
      } else {
        console.log('  ⚠️  No trend data available');
      }
    } else {
      console.log('⚠️  No trends section (expected if no content data)');
    }
  });

  it('should have valid dashboard structure', async () => {
    const result = await client.callTool('content', { time_range: '30d' });

    expect(result.isError).toBe(false);

    const content = result.structuredContent as any;

    console.log('\n✅ Dashboard Structure Validation:');
    console.log('  Title:', content.title);
    console.log('  Description:', content.description);
    console.log('  Sections:', content.sections?.length || 0);
    console.log('  Artifact type:', content['x-artifact-type']);

    // Validate structure
    expect(content['x-artifact-type']).toBe('dashboard');
    expect(content.title).toBeDefined();
    expect(content.description).toBeDefined();
    expect(content.sections).toBeDefined();
    expect(Array.isArray(content.sections)).toBe(true);

    // Validate each section has required fields
    if (content.sections.length > 0) {
      content.sections.forEach((section: any) => {
        expect(section.section_id).toBeDefined();
        expect(section.title).toBeDefined();
        expect(section.section_type).toBeDefined();
        expect(section.data).toBeDefined();
        expect(section.layout).toBeDefined();
      });
    }
  });

  console.log('\n═══════════════════════════════════════════════════════════');
  console.log('✅ MCP TEST SUITE COMPLETE');
  console.log('═══════════════════════════════════════════════════════════\n');
});
