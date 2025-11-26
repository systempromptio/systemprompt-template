export interface AnonymousSession {
  access_token: string;
  token_type: string;
  expires_in: number;
  user_id: string;
  session_id: string;
  client_id: string;
  client_type: string;
}

export interface TestContext {
  contextId: string;
  token: string;
}
