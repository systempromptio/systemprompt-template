// Personal access tokens, one per member, so the access-tokens page has a
// roster to list: live, expired and revoked rows in a fixed mix.
//
// key_hash is a real sha256 of a secret nobody keeps — the page reads the
// prefix and the timestamps, never the hash, and no spec redeems one.
import { createHash } from 'node:crypto';
import type { Client } from 'pg';
import { ID, ago } from './kit';
import { E2E } from './principals';

export async function seedTokens(db: Client) {
  for (const [n, m] of E2E.members.entries()) {
    const expired = n % 3 === 2;
    const revoked = n % 4 === 3;
    const secret = `e2e-secret-${m.id}`;
    await db.query(
      `INSERT INTO user_api_keys
           (id, user_id, name, key_prefix, key_hash, created_at, last_used_at, expires_at, revoked_at)
       VALUES ($1, $2, $3, $4, $5, $6::timestamptz, $7::timestamptz, $8::timestamptz, $9::timestamptz)
       ON CONFLICT (id) DO UPDATE
          SET expires_at = EXCLUDED.expires_at, revoked_at = EXCLUDED.revoked_at`,
      [
        ID.apiKey(n),
        m.id,
        `${m.id.replace('e2e-', '')} laptop`,
        `e2ek${String(n).padStart(4, '0')}`,
        createHash('sha256').update(secret).digest('hex'),
        ago(20 - n),
        n % 2 === 0 ? ago(n, 3) : null,
        expired ? ago(1) : new Date(Date.now() + 30 * 86_400_000),
        revoked ? ago(2) : null,
      ],
    );
  }
}
