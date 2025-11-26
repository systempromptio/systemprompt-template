import { randomUUID } from 'crypto';
import config from '../config';

export interface MessageMetadata {
  sessionId: string;
  traceId: string;
  userId: string;
}

export class AgentClient {
  private baseUrl: string;
  private token: string;

  constructor(baseUrl: string | undefined, token: string) {
    this.baseUrl = baseUrl || `${config.apiBaseUrl}/api/v1/agents`;
    this.token = token;
  }

  async sendMessage(
    agentId: string,
    contextId: string,
    messageText: string,
    metadata: MessageMetadata,
    options?: { stream?: boolean; execute?: boolean }
  ): Promise<{ success: boolean; response?: any; error?: any }> {
    const agentUrl = `${this.baseUrl}/${agentId}`;

    const messageId = randomUUID();
    const stream = options?.stream ?? false;
    const execute = options?.execute ?? false;

    const message = {
      kind: 'message',
      role: 'user',
      parts: [{ kind: 'text', text: messageText }],
      messageId,
      contextId,
    };

    const method = stream ? 'message/stream' : 'message/send';

    const requestBody = {
      jsonrpc: '2.0',
      method,
      params: {
        message,
        configuration: {
          blocking: true,
          execute,
        },
      },
      id: 1,
    };

    try {
      const response = await fetch(agentUrl, {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${this.token}`,
          'Content-Type': 'application/json',
          'x-context-id': contextId,
          'x-trace-id': metadata.traceId,
          'x-session-id': metadata.sessionId,
          'x-user-id': metadata.userId,
        },
        body: JSON.stringify(requestBody),
      });

      if (!response.ok) {
        const errorText = await response.text();
        console.error('Agent request failed:', {
          status: response.status,
          statusText: response.statusText,
          body: errorText,
          url: agentUrl,
          contextId,
          method,
        });

        return {
          success: false,
          error: `HTTP ${response.status}: ${response.statusText} - ${errorText}`,
        };
      }

      const data: any = await response.json();

      if (data.error) {
        console.error('Agent returned error:', data.error);
        return {
          success: false,
          error: data.error,
        };
      }

      return {
        success: true,
        response: data.result,
      };
    } catch (error) {
      console.error('Agent request exception:', error);
      return {
        success: false,
        error: `Request failed: ${error}`,
      };
    }
  }

  // Create EventSource for SSE streaming
  createEventSource(contextId: string): EventSource {
    return new (require('eventsource'))(
      `${config.apiBaseUrl}/api/v1/contexts/${contextId}/events`,
      {
        headers: {
          'Authorization': `Bearer ${this.token}`,
        },
      }
    );
  }
}
