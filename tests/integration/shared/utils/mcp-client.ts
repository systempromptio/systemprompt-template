import { Client } from '@modelcontextprotocol/sdk/client/index.js';
import { StreamableHTTPClientTransport } from '@modelcontextprotocol/sdk/client/streamableHttp.js';
import type { CallToolResult, ListToolsResult } from '@modelcontextprotocol/sdk/types.js';

export interface McpClientOptions {
  baseUrl: string;
  serverName: string;
  token: string;
  contextId?: string;
}

export class McpTestClient {
  private client: Client;
  private transport: StreamableHTTPClientTransport;
  private connected: boolean = false;

  constructor(private options: McpClientOptions) {
    const mcpUrl = `${options.baseUrl}/api/v1/mcp/${options.serverName}/mcp`;

    const headers: Record<string, string> = {
      'Authorization': `Bearer ${options.token}`,
    };

    if (options.contextId) {
      headers['X-Context-ID'] = options.contextId;
    }

    this.transport = new StreamableHTTPClientTransport(new URL(mcpUrl), {
      requestInit: {
        headers,
      }
    });

    this.client = new Client({
      name: 'test-client',
      version: '1.0.0',
    }, {
      capabilities: {},
    });
  }

  async connect(): Promise<void> {
    if (this.connected) {
      return;
    }
    await this.client.connect(this.transport);
    this.connected = true;
  }

  async listTools(): Promise<ListToolsResult> {
    if (!this.connected) {
      await this.connect();
    }
    return await this.client.listTools();
  }

  async callTool(name: string, args: Record<string, any> = {}): Promise<CallToolResult> {
    if (!this.connected) {
      await this.connect();
    }
    return await this.client.callTool({
      name,
      arguments: args,
    });
  }

  async close(): Promise<void> {
    if (this.connected) {
      await this.client.close();
      this.connected = false;
    }
  }
}

export async function createMcpClient(options: McpClientOptions): Promise<McpTestClient> {
  const client = new McpTestClient(options);
  await client.connect();
  return client;
}
