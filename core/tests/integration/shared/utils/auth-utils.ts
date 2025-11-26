import { execSync } from 'child_process';
import { randomUUID } from 'crypto';
import config from '../config';
import type { AnonymousSession } from '../types';

export function generateAdminToken(): string {
  try {
    const output = execSync('just admin-token', {
      encoding: 'utf-8',
      cwd: process.cwd(),
    });

    const lines = output.split('\n');
    const token = lines.find(line => line.startsWith('eyJ'));

    if (!token) {
      throw new Error('Token not found in output');
    }

    return token.trim();
  } catch (error) {
    throw new Error(
      '❌ Failed to generate admin token. No admin user found in database.\n\n' +
      '🔧 Setup required:\n' +
      '   1. Run database migrations: just db migrate\n' +
      '   2. Create an admin user: just assign-admin <username or email>\n\n' +
      `Original error: ${error instanceof Error ? error.message : String(error)}`
    );
  }
}

export function generateTestToken(userId: string = 'test-user', role: string = 'user'): string {
  const header = {
    typ: 'JWT',
    alg: 'HS256',
  };

  const now = Math.floor(Date.now() / 1000);
  const payload = {
    sub: userId,
    iat: now,
    exp: now + 86400, // 24 hours
    iss: config.jwtIssuer,
    aud: ['web', 'a2a', 'api', 'mcp'],
    jti: randomUUID(),
    scope: role === 'admin' ? 'user admin' : 'user',
    username: userId,
    email: `${userId}@test.local`,
    user_type: role,
    token_type: 'Bearer',
    auth_time: now,
    session_id: `sess_${randomUUID()}`,
    rate_limit_tier: role === 'admin' ? 'admin' : 'user',
  };

  const secret = config.jwtSecret;

  const headerEncoded = Buffer.from(JSON.stringify(header)).toString('base64url');
  const payloadEncoded = Buffer.from(JSON.stringify(payload)).toString('base64url');

  const signatureInput = `${headerEncoded}.${payloadEncoded}`;

  const crypto = require('crypto');
  const signature = crypto
    .createHmac('sha256', secret)
    .update(signatureInput)
    .digest('base64url');

  return `${signatureInput}.${signature}`;
}

export function getTokenUserId(token: string): string | null {
  try {
    const parts = token.split('.');
    if (parts.length !== 3) return null;

    const payload = JSON.parse(Buffer.from(parts[1], 'base64').toString());
    return payload.sub || null;
  } catch {
    return null;
  }
}

export function getTokenClaims(token: string): Record<string, any> | null {
  try {
    const parts = token.split('.');
    if (parts.length !== 3) return null;

    return JSON.parse(Buffer.from(parts[1], 'base64').toString());
  } catch {
    return null;
  }
}

export function isTokenExpired(token: string): boolean {
  try {
    const claims = getTokenClaims(token);
    if (!claims || !claims.exp) return true;

    const now = Math.floor(Date.now() / 1000);
    return claims.exp < now;
  } catch {
    return true;
  }
}

// Create unique identifiers for tests
export function createTestId(prefix: string = 'test'): string {
  return `${prefix}-${randomUUID().substring(0, 8)}`;
}

export function createSessionId(): string {
  return `test-${randomUUID()}`;
}

export function createTraceId(): string {
  return randomUUID();
}

export function createContextId(): string {
  return `test-${randomUUID()}`;
}

export async function createContext(token: string, baseUrl: string): Promise<string> {
  const contextId = createContextId();
  const response = await fetch(`${baseUrl}/api/v1/core/contexts`, {
    method: 'POST',
    headers: {
      'Authorization': `Bearer ${token}`,
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({
      context_id: contextId,
      name: `Test Context ${contextId}`,
      metadata: {
        test: true,
      },
    }),
  });

  if (!response.ok) {
    const errorText = await response.text();
    throw new Error(`Failed to create context via API: ${response.status} ${errorText}`);
  }

  const data = await response.json();
  return data.context_id;
}

export async function getAnonymousSession(): Promise<AnonymousSession> {
  // Call the real OAuth anonymous session endpoint
  const response = await fetch(`${config.apiBaseUrl}/api/v1/core/oauth/session`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({
      client_id: 'sp_web',
      metadata: {
        test: true,
        test_id: randomUUID(),
      },
    }),
  });

  if (!response.ok) {
    const errorText = await response.text();
    throw new Error(`Failed to get anonymous session: ${response.status} ${errorText}`);
  }

  const data = await response.json();

  // API returns more fields than we need, just pass through what matches our interface
  return data;
}
