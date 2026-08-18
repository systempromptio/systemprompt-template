/**
 * Confluence user resolution, both directions:
 *   - forward  (publish): display name → account id (`resolveMentions`)
 *   - reverse  (pull):    account id → display name (`makeAccountNameResolver`)
 *
 * The canonical document writes people as plain names; the publisher calls
 * `resolveMentions` to look their account ids up on the fly (Confluence Cloud
 * user search, v1 CQL). Names that cannot be resolved are returned in
 * `unresolved` so the caller can warn — the renderer then falls back to plain
 * text for those.
 *
 * resolveMentions(api, names) → { map: { name: accountId }, unresolved: string[] }
 */

export async function resolveMentions(api, names) {
  const map = {}
  const unresolved = []

  for (const name of names) {
    const id = await lookupAccountId(api, name).catch(() => null)
    if (id) map[name] = id
    else unresolved.push(name)
  }

  return { map, unresolved }
}

async function lookupAccountId(api, name) {
  const cql = `user.fullname~"${name.replace(/"/g, '\\"')}"`
  const data = await api(`search/user?cql=${encodeURIComponent(cql)}&limit=10`, { version: 'v1' })
  const results = data?.results || []

  const users = results
    .map((r) => r.user)
    .filter((u) => u && u.accountId)

  // Prefer an exact (case-insensitive) display-name match; otherwise the first hit.
  const target = name.trim().toLowerCase()
  const exact = users.find((u) => String(u.displayName || u.publicName || '').trim().toLowerCase() === target)
  return (exact || users[0])?.accountId || null
}

/**
 * Build the reverse of `resolveMentions`: an account-id → display-name resolver
 * with a per-instance cache. A page's STORAGE stores a mention as
 * `<ri:user ri:account-id="…"/>` with NO name, so the typed reverse pull resolves
 * the id back to the display name via the v1 user API. The authored doc used the
 * display name ("John Doe"), so we prefer `displayName` over `publicName` (which
 * can be an account handle like "s.rudyi"). Deactivated/inaccessible ids resolve
 * to '' (the cell renders empty) and are cached so we ask at most once per id.
 *
 * @param {(path: string, opts?: object) => Promise<any>} api  the Confluence `api` client
 * @returns {(accountId: string) => Promise<string>}
 */
export function makeAccountNameResolver(api) {
  const cache = new Map()
  return async function resolveAccountName(accountId) {
    if (!accountId) return ''
    if (cache.has(accountId)) return cache.get(accountId)
    let name = ''
    try {
      const data = await api(`user?accountId=${encodeURIComponent(accountId)}`, { version: 'v1' })
      name = data?.displayName || data?.publicName || ''
    } catch {
      // Deactivated/inaccessible users resolve to '' — the cell renders empty.
    }
    cache.set(accountId, name)
    return name
  }
}
