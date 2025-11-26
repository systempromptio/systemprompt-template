import { apiClient } from './api-client'

export interface UserContext {
  context_id: string
  user_id: string
  name: string
  created_at: string
  updated_at: string
}

export interface UserContextWithStats extends UserContext {
  task_count: number
  message_count: number
  last_message_at: string | null
}

class ContextsService {

  async listContexts(authToken: string | null): Promise<{
    contexts?: UserContextWithStats[]
    error?: string
  }> {
    const result = await apiClient.get<UserContextWithStats[]>(
      '/contexts',
      authToken
    )
    return { contexts: result.data, error: result.error }
  }

  async createContext(
    name: string,
    authToken: string | null
  ): Promise<{ context?: UserContext; error?: string }> {
    const result = await apiClient.post<UserContext>(
      '/contexts',
      { name },
      authToken
    )
    return { context: result.data, error: result.error }
  }

  async getContext(
    contextId: string,
    authToken: string | null
  ): Promise<{ context?: UserContext; error?: string }> {
    const result = await apiClient.get<UserContext>(
      `/contexts/${contextId}`,
      authToken
    )
    return { context: result.data, error: result.error }
  }

  async updateContext(
    contextId: string,
    name: string,
    authToken: string | null
  ): Promise<{ context?: UserContext; error?: string }> {
    const result = await apiClient.put<UserContext>(
      `/contexts/${contextId}`,
      { name },
      authToken
    )
    return { context: result.data, error: result.error }
  }

  async deleteContext(
    contextId: string,
    authToken: string | null
  ): Promise<{ success: boolean; error?: string }> {
    const result = await apiClient.delete(
      `/contexts/${contextId}`,
      authToken
    )
    return { success: !result.error, error: result.error }
  }
}

export const contextsService = new ContextsService()
