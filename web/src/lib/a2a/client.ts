import { A2AClient, createAuthenticatingFetchWithRetry, type AuthenticationHandler } from '@a2a-js/sdk/client'
import type {
  AgentCard,
  Part,
  SendMessageResponse,
  GetTaskResponse,
  CancelTaskResponse,
  JSONRPCErrorResponse,
  SendMessageSuccessResponse,
  GetTaskSuccessResponse,
  CancelTaskSuccessResponse,
  Task,
  Message,
  TaskStatusUpdateEvent,
  TaskArtifactUpdateEvent
} from '@a2a-js/sdk'

// A2AStreamEventData type as defined in the SDK but not exported
type A2AStreamEventData = Message | Task | TaskStatusUpdateEvent | TaskArtifactUpdateEvent

export class A2AService {
  private client: A2AClient | null = null
  private agentUrl: string
  private agentCard: AgentCard | null = null
  private _authToken: string | null = null
  private isRefreshingToken: boolean = false
  private refreshPromise: Promise<string | null> | null = null

  constructor(agentUrl: string, authToken?: string | null) {
    this.agentUrl = agentUrl
    this._authToken = authToken || null
  }

  setAuthToken(token: string | null) {
    this._authToken = token
  }

  getAuthToken(): string | null {
    return this._authToken
  }

  resetClient(): void {
    this.client = null
    this.agentCard = null
    this.isRefreshingToken = false
    this.refreshPromise = null
  }

  private async refreshToken(): Promise<string | null> {
    if (this.isRefreshingToken && this.refreshPromise) {
      return this.refreshPromise
    }

    this.isRefreshingToken = true
    this.refreshPromise = this.performTokenRefresh()

    try {
      const newToken = await this.refreshPromise
      return newToken
    } finally {
      this.isRefreshingToken = false
      this.refreshPromise = null
    }
  }

  private async performTokenRefresh(): Promise<string | null> {
    try {
      const { useAuthStore } = await import('@/stores/auth.store')
      const { authService } = await import('@/services/auth.service')

      const userType = useAuthStore.getState().userType

      if (userType === 'anon') {

        const { token, error } = await authService.generateAnonymousToken()

        if (error || !token) {
          useAuthStore.getState().clearAuth()
          return null
        }

        useAuthStore.getState().setAnonymousAuth(
          token.access_token,
          token.user_id,
          token.session_id,
          token.expires_in
        )

        return `Bearer ${token.access_token}`
      } else {
        useAuthStore.getState().clearAuth()
        return null
      }
    } catch (error) {
      return null
    }
  }

  async initialize(existingCard?: AgentCard): Promise<AgentCard> {
    const authHandler: AuthenticationHandler = {
      headers: async () => {
        const headers: Record<string, string> = {}
        if (this._authToken) {
          headers['Authorization'] = this._authToken
        }
        headers['x-trace-id'] = crypto.randomUUID()
        return headers
      },
      shouldRetryWithHeaders: async (_req, res) => {
        if (res.status === 401 || res.status === 403) {
          const responseText = await res.clone().text()

          if (responseText.includes('Invalid or expired JWT token')) {

            const newToken = await this.refreshToken()

            if (newToken) {
              this._authToken = newToken
              return { 'Authorization': newToken }
            }
          } else if (this._authToken) {
            return { 'Authorization': this._authToken }
          }
        }
        return undefined
      }
    }

    const authFetch = createAuthenticatingFetchWithRetry(fetch, authHandler)

    if (existingCard) {
      this.client = new A2AClient(existingCard, { fetchImpl: authFetch })
      this.agentCard = existingCard
      return this.agentCard
    }

    try {
      const cardUrl = `${this.agentUrl}/.well-known/agent-card.json`
      this.client = await A2AClient.fromCardUrl(cardUrl, { fetchImpl: authFetch })
      this.agentCard = await this.client.getAgentCard()
      return this.agentCard
    } catch (error) {
      throw new Error(`Failed to fetch agent card from ${this.agentUrl}/.well-known/agent-card.json: ${error}`)
    }
  }

  async sendMessage(text: string, files?: File[], contextId?: string): Promise<Task | Message | null> {
    if (!this.client) {
      throw new Error('Client not initialized. Please wait for initialization or refresh the page.')
    }

    const parts: Part[] = [{ kind: 'text' as const, text }]

    // Add file parts if present
    if (files?.length) {
      for (const file of files) {
        const bytes = await this.fileToBase64(file)
        parts.push({
          kind: 'file' as const,
          file: {
            name: file.name,
            mimeType: file.type,
            bytes,
          },
        })
      }
    }

    const response = await this.client.sendMessage({
      message: {
        kind: 'message' as const,
        role: 'user' as const,
        parts,
        messageId: crypto.randomUUID() as `${string}-${string}-${string}-${string}-${string}`,
        contextId: contextId as `${string}-${string}-${string}-${string}-${string}` | undefined,
      },
    })

    // Handle the union type response
    if (this.isErrorResponse(response)) {
      throw new Error(`A2A Error: ${response.error.message}`)
    }

    return (response as SendMessageSuccessResponse).result
  }

  async* streamMessage(text: string, contextId?: string, clientMessageId?: string): AsyncGenerator<A2AStreamEventData> {
    if (!this.client) {
      throw new Error('Client not initialized. Please wait for initialization or refresh the page.')
    }

    // A2A spec-compliant: Include pushNotificationConfig in params
    const callbackUrl = contextId
      ? `${window.location.origin}/api/v1/core/contexts/${contextId}/notifications`
      : undefined

    const stream = this.client.sendMessageStream({
      message: {
        kind: 'message' as const,
        role: 'user' as const,
        parts: [{ kind: 'text' as const, text }],
        messageId: crypto.randomUUID() as `${string}-${string}-${string}-${string}-${string}`,
        contextId: contextId as `${string}-${string}-${string}-${string}-${string}` | undefined,
        metadata: clientMessageId ? { clientMessageId } : undefined,
      },
      configuration: callbackUrl ? {
        pushNotificationConfig: {
          url: callbackUrl,
          token: this._authToken || undefined,
        }
      } : undefined,
    })

    for await (const event of stream) {
      yield event
    }
  }

  async getTask(taskId: string): Promise<Task | null> {
    if (!this.client) {
      throw new Error('Client not initialized. Please wait for initialization or refresh the page.')
    }
    const response = await this.client.getTask({ id: taskId })

    // Handle the union type response
    if (this.isErrorResponse(response)) {
      throw new Error(`A2A Error: ${response.error.message}`)
    }

    return (response as GetTaskSuccessResponse).result
  }

  async cancelTask(taskId: string): Promise<Task | null> {
    if (!this.client) {
      throw new Error('Client not initialized. Please wait for initialization or refresh the page.')
    }
    const response = await this.client.cancelTask({ id: taskId })

    // Handle the union type response
    if (this.isErrorResponse(response)) {
      throw new Error(`A2A Error: ${response.error.message}`)
    }

    return (response as CancelTaskSuccessResponse).result
  }

  getAgentCard(): AgentCard | null {
    return this.agentCard
  }

  private isErrorResponse(response: SendMessageResponse | GetTaskResponse | CancelTaskResponse): response is JSONRPCErrorResponse {
    return 'error' in response
  }

  private async fileToBase64(file: File): Promise<string> {
    return new Promise((resolve, reject) => {
      const reader = new FileReader()
      reader.readAsDataURL(file)
      reader.onload = () => {
        const base64 = reader.result as string
        // Remove data URL prefix (e.g., "data:image/png;base64,")
        const base64Data = base64.split(',')[1]
        resolve(base64Data)
      }
      reader.onerror = reject
    })
  }
}

// Singleton instance management
const clients = new Map<string, A2AService>()

export function getA2AClient(
  agentUrl: string,
  authToken?: string | null
): A2AService {
  const cacheKey = agentUrl

  if (!clients.has(cacheKey)) {
    clients.set(cacheKey, new A2AService(agentUrl, authToken))
  } else {
    const client = clients.get(cacheKey)!
    if (authToken !== client.getAuthToken()) {
      client.setAuthToken(authToken || null)
    }
  }
  return clients.get(cacheKey)!
}

export function clearA2AClient(agentUrl: string): void {
  clients.delete(agentUrl)
}