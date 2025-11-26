import { ApiClient } from './api-client';
import { AgentClient } from './agent-client';
import { generateAdminToken, createSessionId, createTraceId } from './auth-utils';
import { TestCleanup } from './db/cleanup';

export interface TestContext {
  token: string;
  apiClient: ApiClient;
  agentClient: AgentClient;
  contextId: string;
  sessionId: string;
  traceId: string;
  cleanup: TestCleanup;
}

export async function createTestContext(name?: string): Promise<TestContext> {
  const token = generateAdminToken();
  const apiClient = new ApiClient(token);
  const agentClient = new AgentClient(undefined, token);
  const sessionId = createSessionId();
  const traceId = createTraceId();
  const cleanup = new TestCleanup();

  const contextName = name || `test-${Date.now()}`;
  const response = await apiClient.createContext(contextName);

  if (response.status !== 200) {
    throw new Error(`Failed to create test context: ${response.error}`);
  }

  return {
    token,
    apiClient,
    agentClient,
    contextId: response.data!.context_id,
    sessionId,
    traceId,
    cleanup,
  };
}

export async function cleanupTestContext(ctx: TestContext): Promise<void> {
  ctx.cleanup.cleanContext(ctx.contextId);
}
