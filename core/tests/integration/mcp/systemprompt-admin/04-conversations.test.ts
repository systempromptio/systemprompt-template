import { describe, it, expect, beforeAll, afterAll } from 'vitest';
import { generateAdminToken, createContext, McpTestClient } from '@test/utils';
import config from '@test/config';

describe('MCP: systemprompt-admin - conversations tool', () => {
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

  it('should list tools and find conversations', async () => {
    const result = await client.listTools();

    console.log('📋 Available admin tools:', JSON.stringify(result.tools.map((t: any) => t.name), null, 2));

    expect(result.tools).toBeDefined();
    const conversationsTool = result.tools.find((t: any) => t.name === 'conversations');
    expect(conversationsTool).toBeDefined();
    expect(conversationsTool.name).toBe('conversations');
    expect(conversationsTool.title).toContain('Conversation');
  });

  it('should execute conversations with default (30d) time range', async () => {
    const input = {};
    console.log('\n💬 Conversations Tool Input:', JSON.stringify(input, null, 2));

    const result = await client.callTool('conversations', input);

    console.log('\n💬 Conversations result (default 30d):');
    console.log('Status:', result.isError ? 'ERROR' : 'SUCCESS');

    if (result.isError) {
      console.log('Error:', JSON.stringify(result, null, 2));
    } else {
      console.log('Content type:', result.content[0]?.type);

      if (result.structuredContent) {
        const conversations = result.structuredContent as any;
        console.log('\n📈 Conversations Dashboard:');
        console.log('  Title:', conversations.title);
        console.log('  Description:', conversations.description);
        console.log('  Sections:', conversations.sections?.length || 0);

        if (conversations.sections) {
          conversations.sections.forEach((section: any) => {
            console.log(`\n  📍 Section: ${section.title}`);
            console.log(`     Type: ${section.section_type}`);
            console.log(`     Data keys: ${Object.keys(section.data || {}).join(', ')}`);

            if (section.section_id === 'conversation_summary' && section.data?.cards) {
              console.log('\n     📊 Conversation Summary Cards:');
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

    const conversations = result.structuredContent as any;
    expect(conversations.title).toBeDefined();
    expect(conversations.sections).toBeDefined();
    expect(conversations.sections.length).toBeGreaterThan(0);
  });

  it('should execute conversations with 7d time range', async () => {
    const input = { time_range: '7d' };
    console.log('\n💬 Conversations Tool Input:', JSON.stringify(input, null, 2));

    const result = await client.callTool('conversations', input);

    console.log('\n💬 Conversations result (7 days):');
    console.log('Status:', result.isError ? 'ERROR' : 'SUCCESS');

    if (result.structuredContent) {
      const conversations = result.structuredContent as any;
      console.log('  Description:', conversations.description);
      console.log('  Sections:', conversations.sections?.length || 0);

      const summarySection = conversations.sections?.find((s: any) => s.section_id === 'conversation_summary');
      if (summarySection?.data?.cards) {
        console.log('\n  📊 Summary Metrics:');
        summarySection.data.cards.forEach((card: any) => {
          console.log(`    ${card.title}: ${card.value}`);
        });
      }
    }

    expect(result.isError).toBe(false);
    expect(result.structuredContent).toBeDefined();
  });

  it('should execute conversations with 30d time range', async () => {
    const input = { time_range: '30d' };
    console.log('\n💬 Conversations Tool Input:', JSON.stringify(input, null, 2));

    const result = await client.callTool('conversations', input);

    console.log('\n💬 Conversations result (30 days):');
    console.log('Status:', result.isError ? 'ERROR' : 'SUCCESS');

    expect(result.isError).toBe(false);
    expect(result.structuredContent).toBeDefined();

    const conversations = result.structuredContent as any;
    expect(conversations.title).toContain('Conversation');
  });

  it('should execute conversations with 90d time range', async () => {
    const input = { time_range: '90d' };
    console.log('\n💬 Conversations Tool Input:', JSON.stringify(input, null, 2));

    const result = await client.callTool('conversations', input);

    console.log('\n💬 Conversations result (90 days):');
    console.log('Status:', result.isError ? 'ERROR' : 'SUCCESS');

    expect(result.isError).toBe(false);
    expect(result.structuredContent).toBeDefined();

    const conversations = result.structuredContent as any;
    expect(conversations.title).toBeDefined();
    expect(conversations.sections).toBeDefined();
  });

  it('should have expected conversation sections', async () => {
    const result = await client.callTool('conversations', {});

    expect(result.isError).toBe(false);

    const conversations = result.structuredContent as any;
    const sectionIds = conversations.sections.map((s: any) => s.section_id);

    console.log('\n✅ Conversation sections found:', sectionIds);

    expect(sectionIds).toContain('conversation_summary');
  });

  it('should show conversations by agent if available', async () => {
    const result = await client.callTool('conversations', {});

    expect(result.isError).toBe(false);

    const conversations = result.structuredContent as any;
    const byAgentSection = conversations.sections?.find((s: any) => s.section_id === 'by_agent');

    if (byAgentSection) {
      console.log('\n🤖 Conversations by Agent:');
      console.log('  Title:', byAgentSection.title);
      console.log('  Type:', byAgentSection.section_type);

      if (byAgentSection.data?.items) {
        byAgentSection.data.items.forEach((item: any) => {
          console.log(`    ${item.label}: ${item.value} conversations (${item.badge} of total)`);
          console.log(`      ${item.secondary}`);
        });
      }
    } else {
      console.log('\n⚠️  No by-agent data available');
    }
  });

  it('should show conversations by status if available', async () => {
    const result = await client.callTool('conversations', {});

    expect(result.isError).toBe(false);

    const conversations = result.structuredContent as any;
    const byStatusSection = conversations.sections?.find((s: any) => s.section_id === 'by_status');

    if (byStatusSection) {
      console.log('\n📊 Conversations by Status:');
      console.log('  Title:', byStatusSection.title);

      if (byStatusSection.data?.items) {
        byStatusSection.data.items.forEach((item: any) => {
          console.log(`    ${item.label}: ${item.value} (${item.badge})`);
        });
      }
    } else {
      console.log('\n⚠️  No by-status data available');
    }
  });

  it('should display summary metrics with proper formatting', async () => {
    const result = await client.callTool('conversations', {});

    expect(result.isError).toBe(false);

    const conversations = result.structuredContent as any;
    const summarySection = conversations.sections?.find((s: any) => s.section_id === 'conversation_summary');

    expect(summarySection).toBeDefined();
    expect(summarySection.data.cards).toBeDefined();
    expect(summarySection.data.cards.length).toBeGreaterThan(0);

    console.log('\n📊 Summary Card Details:');
    summarySection.data.cards.forEach((card: any) => {
      console.log(`  ${card.title}:`);
      console.log(`    Value: ${card.value}`);
      console.log(`    Icon: ${card.icon || 'N/A'}`);
      console.log(`    Status: ${card.status || 'N/A'}`);
    });

    const totalConversationsCard = summarySection.data.cards.find(
      (c: any) => c.title === 'Total Conversations'
    );
    expect(totalConversationsCard).toBeDefined();
  });
});
