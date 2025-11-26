import { ApiClient } from './api-client'

interface AnonymousTokenResponse {
  access_token: string
  token_type: string
  expires_in: number
  session_id: string
  user_id: string
}

interface TokenResponse {
  access_token: string
  token_type: string
  expires_in: number
  refresh_token?: string
  scope?: string
}

class AuthService {
  private oauthClient: ApiClient

  constructor() {
    this.oauthClient = new ApiClient('/api/v1/core/oauth')
  }

  async generateAnonymousToken(): Promise<{
    token?: AnonymousTokenResponse
    error?: string
  }> {
    const result = await this.oauthClient.post<AnonymousTokenResponse>(
      '/session',
      {},
      null
    )

    return { token: result.data, error: result.error }
  }

  async refreshAccessToken(refreshToken: string): Promise<{
    token?: TokenResponse
    error?: string
  }> {
    const body = {
      grant_type: 'refresh_token',
      refresh_token: refreshToken,
    }

    const result = await this.oauthClient.post<TokenResponse>(
      '/token',
      body
    )

    return { token: result.data, error: result.error }
  }
}

export const authService = new AuthService()
