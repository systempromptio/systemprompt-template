import { A2AClient, TaskResult } from '@a2a-js/sdk';
import config from '../config';

export interface MessageMetadata {
  sessionId: string;
  traceId: string;
  userId: string;
}

export class A2AAgentClient {
  private client: A2AClient;
  private agentCardUrl: string;

  constructor(agentCardUrl: string) {
    this.agentCardUrl = agentCardUrl;
    this.client = new A2AClient({
      agentCardUrl,
      fetch: (url, options) => fetch(url, options),
    });
  }

  async sendMessage(
    contextId: string,
    messageText: string,
    metadata: MessageMetadata,
    options?: { stream?: boolean; execute?: boolean }
  ): Promise<{ success: boolean; response?: any; error?: any }> {
    try {
      const message = {
        kind: 'message' as const,
        role: 'user' as const,
        parts: [{ kind: 'text' as const, text: messageText }],
        contextId,
      };

      // Use the A2A client to send the message
      const result = await this.client.task(message, {
        sessionId: metadata.sessionId,
        traceId: metadata.traceId,
        userId: metadata.userId,
      });

      return {
        success: true,
        response: result,
      };
    } catch (error) {
      return {
        success: false,
        error: `Failed to send message: ${error}`,
      };
    }
  }

  async streamMessage(
    contextId: string,
    messageText: string,
    metadata: MessageMetadata,
    options?: { execute?: boolean }
  ): Promise<{ success: boolean; response?: any; error?: any }> {
    try {
      const message = {
        kind: 'message' as const,
        role: 'user' as const,
        parts: [{ kind: 'text' as const, text: messageText }],
        contextId,
      };

      // Stream the message
      const result = await this.client.task(message, {
        sessionId: metadata.sessionId,
        traceId: metadata.traceId,
        userId: metadata.userId,
        stream: true,
      });

      return {
        success: true,
        response: result,
      };
    } catch (error) {
      return {
        success: false,
        error: `Failed to stream message: ${error}`,
      };
    }
  }
}
