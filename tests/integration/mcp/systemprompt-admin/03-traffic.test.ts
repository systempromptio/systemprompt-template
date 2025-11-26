import { describe, it, expect, beforeAll, afterAll } from 'vitest';
import { generateAdminToken, createContext, McpTestClient } from '@test/utils';
import config from '@test/config';

describe('MCP: systemprompt-admin - traffic tool', () => {
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

  it('should list tools and find traffic', async () => {
    const result = await client.listTools();

    console.log('📋 Available admin tools:', JSON.stringify(result.tools.map((t: any) => t.name), null, 2));

    expect(result.tools).toBeDefined();
    const trafficTool = result.tools.find((t: any) => t.name === 'traffic');
    expect(trafficTool).toBeDefined();
    expect(trafficTool.name).toBe('traffic');
    expect(trafficTool.title).toContain('Traffic');
  });

  it('should execute traffic with default (30d) time range', async () => {
    const input = {};
    console.log('\n📊 Traffic Tool Input:', JSON.stringify(input, null, 2));

    const result = await client.callTool('traffic', input);

    console.log('\n📊 Traffic result (default 30d):');
    console.log('Status:', result.isError ? 'ERROR' : 'SUCCESS');

    if (result.isError) {
      console.log('Error:', JSON.stringify(result, null, 2));
    } else {
      console.log('Content type:', result.content[0]?.type);

      if (result.structuredContent) {
        const traffic = result.structuredContent as any;
        console.log('\n📈 Traffic Dashboard:');
        console.log('  Title:', traffic.title);
        console.log('  Description:', traffic.description);
        console.log('  Sections:', traffic.sections?.length || 0);

        if (traffic.sections) {
          traffic.sections.forEach((section: any) => {
            console.log(`\n  📍 Section: ${section.title}`);
            console.log(`     Type: ${section.section_type}`);
            console.log(`     Data keys: ${Object.keys(section.data || {}).join(', ')}`);

            if (section.section_id === 'traffic_summary' && section.data?.cards) {
              console.log('\n     📊 Traffic Summary Cards:');
              section.data.cards.forEach((card: any) => {
                console.log(`       - ${card.title}: ${card.value}`);
              });
            }
          });
        }
      }
    }

    expect(result.isError).toBe(false);
    expect(result.content).toBeDefined();
    expect(result.structuredContent).toBeDefined();

    const traffic = result.structuredContent as any;
    expect(traffic.title).toBeDefined();
    expect(traffic.sections).toBeDefined();
    expect(traffic.sections.length).toBeGreaterThan(0);
  });

  it('should execute traffic with 7d time range', async () => {
    const input = { time_range: '7d' };
    console.log('\n📊 Traffic Tool Input:', JSON.stringify(input, null, 2));

    const result = await client.callTool('traffic', input);

    console.log('\n📊 Traffic result (7 days):');
    console.log('Status:', result.isError ? 'ERROR' : 'SUCCESS');

    if (result.structuredContent) {
      const traffic = result.structuredContent as any;
      console.log('  Description:', traffic.description);
      console.log('  Sections:', traffic.sections?.length || 0);

      const trafficSection = traffic.sections?.find((s: any) => s.section_id === 'traffic_summary');
      if (trafficSection?.data?.cards) {
        console.log('\n  📊 Summary Metrics:');
        trafficSection.data.cards.forEach((card: any) => {
          console.log(`    ${card.title}: ${card.value}`);
        });
      }
    }

    expect(result.isError).toBe(false);
    expect(result.structuredContent).toBeDefined();
  });

  it('should execute traffic with 30d time range', async () => {
    const input = { time_range: '30d' };
    console.log('\n📊 Traffic Tool Input:', JSON.stringify(input, null, 2));

    const result = await client.callTool('traffic', input);

    console.log('\n📊 Traffic result (30 days):');
    console.log('Status:', result.isError ? 'ERROR' : 'SUCCESS');

    expect(result.isError).toBe(false);
    expect(result.structuredContent).toBeDefined();

    const traffic = result.structuredContent as any;
    expect(traffic.title).toContain('Traffic');
  });

  it('should execute traffic with 90d time range', async () => {
    const input = { time_range: '90d' };
    console.log('\n📊 Traffic Tool Input:', JSON.stringify(input, null, 2));

    const result = await client.callTool('traffic', input);

    console.log('\n📊 Traffic result (90 days):');
    console.log('Status:', result.isError ? 'ERROR' : 'SUCCESS');

    expect(result.isError).toBe(false);
    expect(result.structuredContent).toBeDefined();

    const traffic = result.structuredContent as any;
    expect(traffic.title).toBeDefined();
    expect(traffic.sections).toBeDefined();
  });

  it('should have expected traffic sections', async () => {
    const result = await client.callTool('traffic', {});

    expect(result.isError).toBe(false);

    const traffic = result.structuredContent as any;
    const sectionIds = traffic.sections.map((s: any) => s.section_id);

    console.log('\n✅ Traffic sections found:', sectionIds);

    expect(sectionIds).toContain('traffic_summary');
  });

  it('should show device breakdown if available', async () => {
    const result = await client.callTool('traffic', {});

    expect(result.isError).toBe(false);

    const traffic = result.structuredContent as any;
    const deviceSection = traffic.sections?.find((s: any) => s.section_id === 'device_breakdown');

    if (deviceSection) {
      console.log('\n📱 Device Breakdown:');
      console.log('  Title:', deviceSection.title);
      console.log('  Type:', deviceSection.section_type);

      if (deviceSection.data?.items) {
        deviceSection.data.items.forEach((item: any) => {
          console.log(`    ${item.label}: ${item.value} (${item.badge})`);
        });
      }
    } else {
      console.log('\n⚠️  No device breakdown data available');
    }
  });

  it('should show geo breakdown if available', async () => {
    const result = await client.callTool('traffic', {});

    expect(result.isError).toBe(false);

    const traffic = result.structuredContent as any;
    const geoSection = traffic.sections?.find((s: any) => s.section_id === 'geo_breakdown');

    if (geoSection) {
      console.log('\n🌍 Geo Breakdown:');
      console.log('  Title:', geoSection.title);

      if (geoSection.data?.items) {
        geoSection.data.items.forEach((item: any) => {
          console.log(`    ${item.label}: ${item.value} (${item.badge})`);
        });
      }
    } else {
      console.log('\n⚠️  No geo breakdown data available');
    }
  });

  it('should show client breakdown if available', async () => {
    const result = await client.callTool('traffic', {});

    expect(result.isError).toBe(false);

    const traffic = result.structuredContent as any;
    const clientSection = traffic.sections?.find((s: any) => s.section_id === 'client_breakdown');

    if (clientSection) {
      console.log('\n💻 Client Breakdown:');
      console.log('  Title:', clientSection.title);

      if (clientSection.data?.items) {
        clientSection.data.items.forEach((item: any) => {
          console.log(`    ${item.label}: ${item.value} (${item.badge})`);
        });
      }
    } else {
      console.log('\n⚠️  No client breakdown data available');
    }
  });
});
