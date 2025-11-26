import { describe, it, expect, beforeAll, afterAll } from 'vitest';
import { generateAdminToken, createContext, McpTestClient } from '@test/utils';
import config from '@test/config';

describe('MCP: systemprompt-admin - dashboard tool', () => {
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

  it('should list tools and find dashboard', async () => {
    const result = await client.listTools();

    console.log('📋 Available admin tools:', JSON.stringify(result.tools, null, 2));

    expect(result.tools).toBeDefined();
    const dashboardTool = result.tools.find((t: any) => t.name === 'dashboard');
    expect(dashboardTool).toBeDefined();
    expect(dashboardTool.name).toBe('dashboard');
    expect(dashboardTool.title).toContain('Dashboard');
  });

  it('should execute dashboard with default (24h) time range', async () => {
    const result = await client.callTool('dashboard', {});

    console.log('\n📊 Dashboard result (default 24h):');
    console.log('Status:', result.isError ? 'ERROR' : 'SUCCESS');
    console.log('Full result:', JSON.stringify(result, null, 2));

    if (result.isError) {
      console.log('Error:', JSON.stringify(result, null, 2));
    } else {
      console.log('Content type:', result.content[0]?.type);

      if (result.structuredContent) {
        const dashboard = result.structuredContent as any;
        console.log('\n📈 Dashboard sections:');
        console.log('  Title:', dashboard.title);
        console.log('  Sections:', dashboard.sections?.length || 0);

        if (dashboard.sections) {
          dashboard.sections.forEach((section: any) => {
            console.log(`\n  📍 Section: ${section.title}`);
            console.log(`     Type: ${section.section_type}`);
            console.log(`     Data keys: ${Object.keys(section.data || {}).join(', ')}`);
          });
        }
      } else {
        console.log('WARNING: structuredContent is undefined');
      }
    }

    expect(result.isError).toBe(false);
    expect(result.content).toBeDefined();
    expect(result.structuredContent).toBeDefined();

    const dashboard = result.structuredContent as any;
    expect(dashboard.title).toBeDefined();
    expect(dashboard.sections).toBeDefined();
    expect(dashboard.sections.length).toBeGreaterThan(0);
  });

  it('should execute dashboard with realtime time range', async () => {
    const result = await client.callTool('dashboard', { time_range: 'realtime' });

    console.log('\n📊 Dashboard result (realtime):');
    console.log('Status:', result.isError ? 'ERROR' : 'SUCCESS');

    if (result.structuredContent) {
      const dashboard = result.structuredContent as any;
      console.log('  Description:', dashboard.description);
      console.log('  Sections:', dashboard.sections?.length || 0);
    }

    expect(result.isError).toBe(false);
    expect(result.structuredContent).toBeDefined();
  });

  it('should execute dashboard with 7d time range', async () => {
    const result = await client.callTool('dashboard', { time_range: '7d' });

    console.log('\n📊 Dashboard result (7 days):');
    console.log('Status:', result.isError ? 'ERROR' : 'SUCCESS');

    expect(result.isError).toBe(false);
    expect(result.structuredContent).toBeDefined();

    const dashboard = result.structuredContent as any;
    expect(dashboard.title).toContain('Dashboard');
  });

  it('should execute dashboard with 30d time range', async () => {
    const result = await client.callTool('dashboard', { time_range: '30d' });

    console.log('\n📊 Dashboard result (30 days):');
    console.log('Status:', result.isError ? 'ERROR' : 'SUCCESS');

    expect(result.isError).toBe(false);
    expect(result.structuredContent).toBeDefined();

    const dashboard = result.structuredContent as any;
    expect(dashboard.title).toBeDefined();
    expect(dashboard.sections).toBeDefined();
  });

  it('should have all expected dashboard sections', async () => {
    const result = await client.callTool('dashboard', {});

    expect(result.isError).toBe(false);

    const dashboard = result.structuredContent as any;
    const sectionIds = dashboard.sections.map((s: any) => s.section_id);

    console.log('\n✅ Dashboard sections found:', sectionIds);

    expect(sectionIds).toContain('realtime_activity');
    expect(sectionIds).toContain('agent_conversations');
    expect(sectionIds).toContain('system_health');
  });
});
