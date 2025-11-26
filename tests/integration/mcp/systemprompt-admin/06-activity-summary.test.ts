import { describe, it, expect, beforeAll, afterAll } from 'vitest';
import { generateAdminToken, createContext, McpTestClient } from '@test/utils';
import config from '@test/config';

describe('MCP: systemprompt-admin - activity_summary tool', () => {
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

  it('should list tools and find activity_summary', async () => {
    const result = await client.listTools();

    console.log('📋 Available admin tools:', result.tools.map((t: any) => t.name).join(', '));

    expect(result.tools).toBeDefined();
    const activityTool = result.tools.find((t: any) => t.name === 'activity_summary');
    expect(activityTool).toBeDefined();
    expect(activityTool.name).toBe('activity_summary');
    expect(activityTool.title).toContain('Activity Summary');
  });

  it('should execute activity_summary and return dashboard', async () => {
    const result = await client.callTool('activity_summary', {});

    console.log('\n📊 Activity Summary Dashboard Result:');
    console.log('Status:', result.isError ? 'ERROR' : 'SUCCESS');

    if (result.isError) {
      console.log('Error:', JSON.stringify(result, null, 2));
    } else {
      console.log('Content type:', result.content[0]?.type);

      if (result.content && result.content[0]) {
        const textContent = result.content[0] as any;
        if (textContent.text) {
          console.log('\n📝 Text Content (compact):');
          console.log(textContent.text);
        }
      }

      if (result.structuredContent) {
        const dashboard = result.structuredContent as any;
        console.log('\n🎨 Dashboard Structured Content:');
        console.log('  Artifact Type:', dashboard['x-artifact-type']);
        console.log('  Title:', dashboard.title);
        console.log('  Description:', dashboard.description);
        console.log('  Sections:', dashboard.sections?.length || 0);

        if (dashboard.sections) {
          dashboard.sections.forEach((section: any, idx: number) => {
            console.log(`\n  📍 Section ${idx + 1}: ${section.title}`);
            console.log(`     ID: ${section.section_id}`);
            console.log(`     Type: ${section.section_type}`);
            console.log(`     Data keys: ${Object.keys(section.data || {}).join(', ')}`);

            if (section.section_type === 'metrics_cards' && section.data?.cards) {
              console.log(`     Cards: ${section.data.cards.length}`);
              section.data.cards.forEach((card: any) => {
                console.log(`       - ${card.title}: ${card.value} (${card.subtitle})`);
              });
            }

            if (section.section_type === 'table' && section.data?.items) {
              console.log(`     Rows: ${section.data.items.length}`);
              console.log(`     Columns: ${section.data.columns?.map((c: any) => c.header).join(', ')}`);
              if (section.data.items.length > 0) {
                console.log(`     First row:`, JSON.stringify(section.data.items[0], null, 2));
              }
            }
          });
        }
      } else {
        console.log('WARNING: structuredContent is undefined');
      }
    }

    expect(result.isError).toBe(false);
    expect(result.content).toBeDefined();
    expect(result.content.length).toBeGreaterThan(0);
    expect(result.structuredContent).toBeDefined();

    const dashboard = result.structuredContent as any;
    expect(dashboard['x-artifact-type']).toBe('dashboard');
    expect(dashboard.title).toBe('Activity Summary');
    expect(dashboard.sections).toBeDefined();
    expect(dashboard.sections.length).toBe(4);
  });

  it('should have metrics cards section with 24h highlights', async () => {
    const result = await client.callTool('activity_summary', {});

    expect(result.isError).toBe(false);

    const dashboard = result.structuredContent as any;
    const cardsSection = dashboard.sections.find((s: any) => s.section_id === 'metrics_cards');

    expect(cardsSection).toBeDefined();
    expect(cardsSection.section_type).toBe('metrics_cards');
    expect(cardsSection.title).toBe('Key Metrics');
    expect(cardsSection.data.cards).toBeDefined();
    expect(cardsSection.data.cards.length).toBe(4);

    const cardTitles = cardsSection.data.cards.map((c: any) => c.title);
    expect(cardTitles).toContain('Conversations');
    expect(cardTitles).toContain('Active Users');
    expect(cardTitles).toContain('Tool Executions');
    expect(cardTitles).toContain('New Users');

    console.log('\n✅ Metrics cards section validated');
    console.log('   Cards:', cardTitles.join(', '));
  });

  it('should have main metrics table with all time periods', async () => {
    const result = await client.callTool('activity_summary', {});

    expect(result.isError).toBe(false);

    const dashboard = result.structuredContent as any;
    const mainTableSection = dashboard.sections.find((s: any) => s.section_id === 'main_metrics');

    expect(mainTableSection).toBeDefined();
    expect(mainTableSection.section_type).toBe('table');
    expect(mainTableSection.title).toBe('Platform Metrics');
    expect(mainTableSection.data.columns).toBeDefined();
    expect(mainTableSection.data.items).toBeDefined();
    expect(mainTableSection.data.items.length).toBe(6);

    const metrics = mainTableSection.data.items.map((r: any) => r.metric);
    expect(metrics).toContain('Conversations');
    expect(metrics).toContain('Active Users (with conversations)');
    expect(metrics).toContain('Active Users (with tool usage)');
    expect(metrics).toContain('Active Users (combined)');
    expect(metrics).toContain('New Users');
    expect(metrics).toContain('Tool Executions');

    console.log('\n✅ Main metrics table validated');
    console.log('   Metrics:', metrics.join(', '));
    console.log('\n📊 Sample data from main table:');
    mainTableSection.data.items.forEach((row: any) => {
      console.log(`   ${row.metric}: 24h=${row.h24}, 7d=${row.d7}, 31d=${row.d31}`);
    });
  });

  it('should have tool usage by agent table', async () => {
    const result = await client.callTool('activity_summary', {});

    expect(result.isError).toBe(false);

    const dashboard = result.structuredContent as any;
    const agentTableSection = dashboard.sections.find((s: any) => s.section_id === 'tool_usage_by_agent');

    expect(agentTableSection).toBeDefined();
    expect(agentTableSection.section_type).toBe('table');
    expect(agentTableSection.title).toBe('Tool Usage by Agent');
    expect(agentTableSection.data.columns).toBeDefined();
    expect(agentTableSection.data.items).toBeDefined();

    console.log('\n✅ Tool usage by agent table validated');
    console.log(`   Agents found: ${agentTableSection.data.items.length}`);
    if (agentTableSection.data.items.length > 0) {
      console.log('\n📊 Top agents by tool usage:');
      agentTableSection.data.items.slice(0, 5).forEach((row: any) => {
        console.log(`   ${row.agent}: 24h=${row.h24}, 7d=${row.d7}, 31d=${row.d31}`);
      });
    }
  });

  it('should have tool usage by tool name table', async () => {
    const result = await client.callTool('activity_summary', {});

    expect(result.isError).toBe(false);

    const dashboard = result.structuredContent as any;
    const toolTableSection = dashboard.sections.find((s: any) => s.section_id === 'tool_usage_by_tool');

    expect(toolTableSection).toBeDefined();
    expect(toolTableSection.section_type).toBe('table');
    expect(toolTableSection.title).toBe('Tool Usage by Tool Name');
    expect(toolTableSection.data.columns).toBeDefined();
    expect(toolTableSection.data.items).toBeDefined();

    console.log('\n✅ Tool usage by tool name table validated');
    console.log(`   Tools found: ${toolTableSection.data.items.length}`);
    if (toolTableSection.data.items.length > 0) {
      console.log('\n📊 Top tools by usage:');
      toolTableSection.data.items.slice(0, 5).forEach((row: any) => {
        console.log(`   ${row.tool}: 24h=${row.h24}, 7d=${row.d7}, 31d=${row.d31}`);
      });
    }
  });

  it('should have numeric values for all metrics', async () => {
    const result = await client.callTool('activity_summary', {});

    expect(result.isError).toBe(false);

    const dashboard = result.structuredContent as any;
    const mainTableSection = dashboard.sections.find((s: any) => s.section_id === 'main_metrics');

    mainTableSection.data.items.forEach((row: any) => {
      expect(typeof row.h24).toBe('number');
      expect(typeof row.d7).toBe('number');
      expect(typeof row.d31).toBe('number');
      expect(row.h24).toBeGreaterThanOrEqual(0);
      expect(row.d7).toBeGreaterThanOrEqual(0);
      expect(row.d31).toBeGreaterThanOrEqual(0);
    });

    console.log('\n✅ All metrics have valid numeric values');
  });

  it('should have compact text content alongside structured dashboard', async () => {
    const result = await client.callTool('activity_summary', {});

    expect(result.isError).toBe(false);

    const textContent = result.content[0] as any;
    const dashboard = result.structuredContent as any;

    expect(textContent.text).toBeDefined();
    expect(typeof textContent.text).toBe('string');
    expect(textContent.text.length).toBeGreaterThan(0);
    expect(textContent.text).toContain('Activity summary dashboard generated');

    expect(dashboard['x-artifact-type']).toBe('dashboard');

    console.log('\n✅ Both compact text and rich dashboard present');
    console.log(`   Text: "${textContent.text}"`);
    console.log(`   Dashboard: ${dashboard.sections.length} sections`);
  });
});
