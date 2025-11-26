import config from '../config';
import { withRetry, delay } from './retry-utils';

export interface AgentInfo {
  protocolVersion: string;
  name: string;
  description: string;
  url: string;
  version: string;
  preferredTransport: string;
  capabilities: {
    streaming: boolean;
    pushNotifications: boolean;
    stateTransitionHistory: boolean;
    extensions?: any[];
  };
}

export class AgentRegistry {
  private baseUrl: string;
  private static cachedAgents: AgentInfo[] | null = null;

  constructor(baseUrl: string = config.apiBaseUrl) {
    this.baseUrl = baseUrl;
  }

  static async getRunningAgentNames(): Promise<string[]> {
    const registry = new AgentRegistry();
    const agents = await registry.listAgents();
    return agents.map(a => a.name);
  }

  static async hasAgent(name: string): Promise<boolean> {
    const registry = new AgentRegistry();
    return await registry.isAgentAvailable(name);
  }

  static clearCache(): void {
    this.cachedAgents = null;
  }

  async listAgents(): Promise<AgentInfo[]> {
    return withRetry(
      async () => {
        const response = await fetch(`${this.baseUrl}/api/v1/agents/registry`, {
          headers: {
            'Content-Type': 'application/json',
          },
        });

        // Check for rate limiting
        if (response.status === 429) {
          await delay(1000);
          throw new Error('Rate limited');
        }

        if (!response.ok) {
          throw new Error(`Failed to fetch agent registry: ${response.status}`);
        }

        const data = await response.json();
        return data.data || [];
      },
      3,    // max attempts
      1000, // initial delay
      1.5   // backoff multiplier
    );
  }

  async getAgent(agentName: string): Promise<AgentInfo | null> {
    const agents = await this.listAgents();
    return agents.find(a => a.name === agentName) || null;
  }

  async getAgentUrl(agentName: string): Promise<string | null> {
    const agent = await this.getAgent(agentName);
    return agent?.url || null;
  }

  async isAgentAvailable(agentName: string): Promise<boolean> {
    const agent = await this.getAgent(agentName);
    return agent !== null;
  }
}
