import config from '../config';
import { withRetry, delay } from './retry-utils';

export interface Task {
  id: string;
  status: {
    state: string;
    reason?: string;
  };
  created_at: string;
  updated_at: string;
  history: any[];
  user_id: string;
  session_id: string;
  trace_id: string;
  context_id?: string;
}

export interface Artifact {
  artifact_id: string;
  task_uuid: string;
  parts: any[];
  metadata: Record<string, any>;
  created_at: string;
}

export interface Context {
  context_id: string;
  name: string;
  created_at: string;
  updated_at: string;
}

export class ApiClient {
  private baseUrl: string;
  private token?: string;

  constructor(token?: string, baseUrl: string = config.apiBaseUrl) {
    this.baseUrl = baseUrl;
    this.token = token;
  }

  private async request<T>(
    endpoint: string,
    options: RequestInit = {}
  ): Promise<{ status: number; data?: T; error?: any; headers: Record<string, string> }> {
    return withRetry(
      async () => {
        const url = `${this.baseUrl}${endpoint}`;
        const headers: Record<string, string> = {
          'Content-Type': 'application/json',
          ...options.headers,
        };

        if (this.token) {
          headers['Authorization'] = `Bearer ${this.token}`;
        }

        const response = await fetch(url, { ...options, headers });

        // Check for rate limiting
        if (response.status === 429) {
          // Wait before retrying
          await delay(1000);
          throw new Error('Rate limited');
        }

        const data = await response.json().catch(() => null);

        const responseHeaders: Record<string, string> = {};
        response.headers.forEach((value, key) => {
          responseHeaders[key] = value;
        });

        return {
          status: response.status,
          data: response.ok ? (data as T) : undefined,
          error: !response.ok ? data : undefined,
          headers: responseHeaders,
        };
      },
      3,  // max attempts
      1000, // initial delay
      1.5   // backoff multiplier
    );
  }

  async getTasks(options?: {
    contextId?: string;
    status?: string;
    limit?: number;
  }): Promise<{ status: number; data?: Task[]; error?: any }> {
    if (options?.contextId) {
      return this.request<Task[]>(
        `/api/v1/core/contexts/${options.contextId}/tasks`
      );
    }

    const params = new URLSearchParams();
    if (options?.status) params.append('status', options.status);
    if (options?.limit) params.append('limit', options.limit.toString());

    const query = params.toString() ? `?${params.toString()}` : '';
    return this.request<Task[]>(`/api/v1/core/tasks${query}`);
  }

  async getTask(taskId: string): Promise<{ status: number; data?: Task; error?: any }> {
    return this.request<Task>(`/api/v1/core/tasks/${taskId}`);
  }

  async getArtifacts(options?: {
    contextId?: string;
    taskId?: string;
  }): Promise<{ status: number; data?: Artifact[]; error?: any }> {
    if (options?.contextId) {
      return this.request<Artifact[]>(
        `/api/v1/core/contexts/${options.contextId}/artifacts`
      );
    }

    if (options?.taskId) {
      return this.request<Artifact[]>(
        `/api/v1/core/tasks/${options.taskId}/artifacts`
      );
    }

    throw new Error('Must provide contextId or taskId');
  }

  async getArtifact(artifactId: string): Promise<{ status: number; data?: Artifact; error?: any }> {
    return this.request<Artifact>(`/api/v1/core/artifacts/${artifactId}`);
  }

  async createContext(name: string): Promise<{ status: number; data?: Context; error?: any }> {
    return this.request<Context>('/api/v1/core/contexts', {
      method: 'POST',
      body: JSON.stringify({ name }),
    });
  }

  async getContexts(): Promise<{ status: number; data?: { contexts: Context[] }; error?: any }> {
    const result = await this.request<{ detail: { contexts: Context[] } }>('/api/v1/core/contexts');
    if (result.data?.detail) {
      return {
        status: result.status,
        data: { contexts: result.data.detail.contexts },
      };
    }
    return result as any;
  }

  async getContext(contextId: string): Promise<{ status: number; data?: Context; error?: any }> {
    return this.request<Context>(`/api/v1/core/contexts/${contextId}`);
  }

  async get<T = any>(endpoint: string, options?: { headers?: Record<string, string> }): Promise<{ status: number; data?: T; error?: any; headers: Record<string, string> }> {
    return this.request<T>(endpoint, {
      method: 'GET',
      headers: options?.headers,
    });
  }

  async post<T = any>(endpoint: string, body?: any, options?: { headers?: Record<string, string> }): Promise<{ status: number; data?: T; error?: any; headers: Record<string, string> }> {
    return this.request<T>(endpoint, {
      method: 'POST',
      body: body ? JSON.stringify(body) : undefined,
      headers: options?.headers,
    });
  }
}
