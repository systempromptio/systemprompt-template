import '../../../../../../.project/lib/load-env.mjs'

const email = process.env.ATLASSIAN_EMAIL
const token = process.env.ATLASSIAN_API_TOKEN
// Strip any trailing slash so URL building (`${baseUrl}/wiki/...`) never doubles it.
const baseUrl = (process.env.ATLASSIAN_BASE_URL || '').replace(/\/+$/, '')

if (!email || !token || !baseUrl) {
  console.error('ERROR: ATLASSIAN_EMAIL, ATLASSIAN_API_TOKEN, and ATLASSIAN_BASE_URL must be set.')
  console.error('Run: node .cursor/.project/init.mjs — then fill in credentials in .cursor/.project/.env')
  console.error('Get your API token at: https://id.atlassian.com/manage-profile/security/api-tokens')
  process.exit(1)
}

if (token === 'your-api-token-here') {
  console.error('ERROR: Replace the placeholder in .cursor/.project/.env with your real ATLASSIAN_API_TOKEN.')
  process.exit(1)
}

const AUTH = `Basic ${Buffer.from(`${email}:${token}`).toString('base64')}`

export { baseUrl, AUTH }

/**
 * Core authenticated JSON request against any absolute URL. Every product client
 * below is a thin base-URL adapter over this: build the URL, delegate here.
 * Returns parsed JSON, `null` on 204, and throws a formatted error on non-2xx.
 */
export async function fetchJson(url, { method = 'GET', body } = {}) {
  const options = {
    method,
    headers: {
      'Authorization': AUTH,
      'Accept': 'application/json',
    },
  }

  if (body) {
    options.headers['Content-Type'] = 'application/json'
    options.body = typeof body === 'string' ? body : JSON.stringify(body)
  }

  const res = await fetch(url, options)

  if (res.status === 204) return null

  const data = await res.json().catch(() => null)

  if (!res.ok) {
    throw new Error(`HTTP ${res.status}: ${formatApiError(data, res.statusText)}`)
  }

  return data
}

/**
 * Confluence Cloud client. `version: 'v2'` (default) hits `/wiki/api/v2/<path>`;
 * `version: 'v1'` hits `/wiki/rest/api/<path>`. An absolute URL passes through.
 */
export async function api(path, { method = 'GET', body, version = 'v2' } = {}) {
  const url = path.startsWith('http')
    ? path
    : `${baseUrl}/wiki/${version === 'v1' ? 'rest/api' : 'api/v2'}/${path}`
  return fetchJson(url, { method, body })
}

/** Jira Cloud platform REST v3 client (`/rest/api/3/<path>`). */
export async function jiraApi(path, { method = 'GET', body } = {}) {
  return fetchJson(`${baseUrl}/rest/api/3/${path}`, { method, body })
}

/** Jira Agile REST client (`/rest/agile/1.0/<path>`). */
export async function agileApi(path, { method = 'GET', body } = {}) {
  return fetchJson(`${baseUrl}/rest/agile/1.0/${path}`, { method, body })
}

function formatApiError(data, fallback) {
  const parts = []
  if (data?.errorMessages?.length) parts.push(data.errorMessages.join(', '))
  if (Array.isArray(data?.errors)) {
    // Confluence Cloud v2: errors = [{ status, code, title, detail }]
    const arr = data.errors
      .map((e) => (typeof e === 'string' ? e : [e.title, e.detail].filter(Boolean).join(' — ') || e.code))
      .filter(Boolean)
      .join('; ')
    if (arr) parts.push(arr)
  } else if (data?.errors && typeof data.errors === 'object') {
    const fieldErrors = Object.entries(data.errors)
      .map(([field, msg]) => `${field}: ${typeof msg === 'string' ? msg : JSON.stringify(msg)}`)
      .join('; ')
    if (fieldErrors) parts.push(fieldErrors)
  }
  if (data?.message) parts.push(data.message)
  return parts.join(' | ') || fallback
}

export function die(msg) {
  console.error(`ERROR: ${msg}`)
  process.exit(1)
}
