// Mints admin-plane JWTs the same way core's JwtService::generate_admin_token
// does, signing with the server's own signing_key.pem. Roles inside the token
// are IGNORED by the server — users.roles in the database decides per request —
// so one claim shape serves every principal; only `sub`/`email` vary.
import { createPrivateKey, randomUUID } from 'node:crypto';
import { readFileSync } from 'node:fs';
import { SignJWT, importPKCS8 } from 'jose';

export interface MintParams {
  userId: string;
  email: string;
  issuer: string;
  kid: string;
  pemPath: string;
  /** Must name a `user_sessions` row belonging to `userId` — the server
   *  attests this and replaces an unattested cookie with an anonymous one. */
  sessionId: string;
  ttlSeconds?: number;
}

export async function mintToken(p: MintParams): Promise<string> {
  const pem = readFileSync(p.pemPath, 'utf8');
  // signing_key.pem may be PKCS#1 ("BEGIN RSA PRIVATE KEY"); jose only imports
  // PKCS#8, so normalize through node:crypto first.
  const pkcs8 = createPrivateKey(pem).export({ type: 'pkcs8', format: 'pem' }).toString();
  const key = await importPKCS8(pkcs8, 'RS256');

  const now = Math.floor(Date.now() / 1000);
  const ttl = p.ttlSeconds ?? 2 * 60 * 60;

  return new SignJWT({
    nbf: now,
    aud: ['web', 'api', 'a2a', 'mcp'],
    jti: randomUUID(),
    scope: 'admin',
    username: p.email,
    email: p.email,
    user_type: 'admin',
    roles: ['admin', 'user'],
    token_type: 'Bearer',
    auth_time: now,
    session_id: p.sessionId,
    rate_limit_tier: 'admin',
  })
    .setProtectedHeader({ alg: 'RS256', kid: p.kid })
    .setSubject(p.userId)
    .setIssuedAt(now)
    .setExpirationTime(now + ttl)
    .setIssuer(p.issuer)
    .sign(key);
}
