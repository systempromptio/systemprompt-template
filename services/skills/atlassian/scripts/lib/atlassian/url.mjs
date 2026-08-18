/**
 * Confluence URL building + space-key resolution — the ONE home for the
 * `${baseUrl}/wiki/...` string shapes that used to be re-derived inline in
 * confluence.mjs, doc/publish.mjs and atlassian/attachments.mjs.
 *
 * Responsibility: turn ids/keys into fetchable/human URLs and resolve a page's
 *   space key. Depends on auth.mjs (baseUrl + api client), so importing this
 *   loads credentials — fine for the real-publish/CLI paths, but keep it OUT of
 *   the --dry code path (lazy-import it after the dry early-return instead).
 * Edit here when: the Confluence URL layout or space lookup changes.
 */

import { baseUrl, api } from './auth.mjs'

/** `${baseUrl}/wiki` — the v1 REST + attachment host root. */
export const wikiBase = () => `${baseUrl}/wiki`

/** Human page URL: `${baseUrl}/wiki/spaces/<KEY>/pages/<ID>`. */
export function pageUrl(spaceKey, pageId) {
  return `${baseUrl}/wiki/spaces/${spaceKey}/pages/${pageId}`
}

// Alias kept for the confluence.mjs command call sites that print page URLs.
export const buildPageUrl = pageUrl

/** Resolve a page's numeric spaceId to its human space key. */
export async function getSpaceKeyForPage(pageId) {
  const page = await api(`pages/${pageId}`)
  const space = await api(`spaces/${page.spaceId}`)
  return space.key
}
