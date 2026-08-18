import { readFileSync, writeFileSync } from 'node:fs'
import { basename, extname } from 'node:path'
import { api, baseUrl, AUTH } from './auth.mjs'
import { wikiBase } from './url.mjs'

const MIME_BY_EXT = {
  '.png': 'image/png',
  '.jpg': 'image/jpeg',
  '.jpeg': 'image/jpeg',
  '.gif': 'image/gif',
  '.svg': 'image/svg+xml',
  '.webp': 'image/webp',
  '.drawio': 'application/vnd.jgraph.mxfile',
}

/**
 * Best-effort content type from a filename extension. PNG/SVG/etc. get real
 * image MIME types so Confluence renders them inline crisply; unknown types
 * fall back to a generic binary type.
 */
export function mimeForFile(fileName) {
  return MIME_BY_EXT[extname(fileName).toLowerCase()] || 'application/octet-stream'
}

export async function listAttachments(pageId) {
  const data = await api(`pages/${pageId}/attachments?limit=250`)
  return data.results || []
}

/**
 * Read the upload comment stored on an attachment. The comment is where the
 * image-sync layer stashes the `sha256:<hex>` gate. Reads the field from the
 * v2 list object when present; otherwise falls back to the v1 attachment
 * metadata. Returns null when no comment can be resolved.
 */
export async function readStoredComment(attachment) {
  if (!attachment) return null
  if (typeof attachment.comment === 'string') return attachment.comment
  if (typeof attachment.metadata?.comment === 'string') return attachment.metadata.comment
  if (typeof attachment.version?.message === 'string') return attachment.version.message
  try {
    const data = await api(`content/${attachment.id}?expand=metadata.comment`, { version: 'v1' })
    return data?.metadata?.comment ?? null
  } catch {
    return null
  }
}

/** Resolve an attachment's binary download URL to an absolute, fetchable URL. */
function resolveDownloadUrl(link) {
  if (!link) return null
  if (link.startsWith('http')) return link
  if (link.startsWith('/wiki/')) return `${baseUrl}${link}`
  return `${baseUrl}/wiki${link.startsWith('/') ? '' : '/'}${link}`
}

/**
 * Download an attachment's binary content to `destPath`.
 *
 * Accepts either a v2/v1 attachment object (uses its `downloadLink` /
 * `_links.download`) or a plain file name (looked up on the page by title).
 * Fetches with the shared `AUTH` so private-space attachments resolve.
 *
 * @param {string} pageId
 * @param {string|object} fileNameOrAttachment
 * @param {string} destPath
 * @returns {Promise<{ fileName: string, destPath: string }>}
 */
export async function downloadAttachment(pageId, fileNameOrAttachment, destPath) {
  let att = fileNameOrAttachment
  if (typeof fileNameOrAttachment === 'string') {
    att = (await listAttachments(pageId)).find((a) => a.title === fileNameOrAttachment)
    if (!att) throw new Error(`attachment "${fileNameOrAttachment}" not found on page ${pageId}`)
  }

  const url = resolveDownloadUrl(att.downloadLink || att._links?.download)
  if (!url) throw new Error(`attachment "${att.title}" has no download link`)

  const res = await fetch(url, { headers: { Authorization: AUTH, Accept: '*/*' } })
  if (!res.ok) throw new Error(`HTTP ${res.status}: failed to download ${att.title}`)

  writeFileSync(destPath, Buffer.from(await res.arrayBuffer()))
  return { fileName: att.title, destPath }
}

export async function uploadAttachment(pageId, filePath, comment = 'Attachment uploaded via script') {
  const fileName = basename(filePath)
  const fileBuf = readFileSync(filePath)

  const form = new FormData()
  form.append('file', new Blob([fileBuf], { type: mimeForFile(fileName) }), fileName)
  form.append('comment', comment)

  const res = await fetch(`${wikiBase()}/rest/api/content/${pageId}/child/attachment`, {
    method: 'POST',
    headers: { Authorization: AUTH, 'X-Atlassian-Token': 'no-check' },
    body: form,
  })

  const data = await res.json().catch(() => null)
  if (!res.ok) {
    throw new Error(`HTTP ${res.status}: ${data?.message || 'attachment upload failed'}`)
  }

  const att = data.results?.[0]
  return { id: att?.id, title: att?.title || fileName, created: true }
}

export async function updateAttachment(pageId, attachmentId, filePath, comment = 'Attachment updated via script') {
  const fileName = basename(filePath)
  const fileBuf = readFileSync(filePath)

  const form = new FormData()
  form.append('file', new Blob([fileBuf], { type: mimeForFile(fileName) }), fileName)
  form.append('comment', comment)

  const res = await fetch(
    `${wikiBase()}/rest/api/content/${pageId}/child/attachment/${attachmentId}/data`,
    {
      method: 'POST',
      headers: { Authorization: AUTH, 'X-Atlassian-Token': 'no-check' },
      body: form,
    }
  )

  const data = await res.json().catch(() => null)
  if (!res.ok) {
    throw new Error(`HTTP ${res.status}: ${data?.message || 'attachment update failed'}`)
  }

  const att = data.results?.[0]
  return { id: att?.id || attachmentId, title: att?.title || fileName, created: false }
}

/**
 * Delete an attachment by its content id (v1 REST — the v2 API has no attachment
 * delete). Used by the publish-time prune of managed orphans (images/diagram
 * sources no longer referenced by the document). `pageId` is accepted for symmetry
 * and logging; the delete is keyed by the attachment's own id.
 *
 * @param {string} pageId
 * @param {string} attachmentId  the attachment content id (e.g. `att123` / numeric)
 * @returns {Promise<{ id: string, deleted: true }>}
 */
export async function deleteAttachment(pageId, attachmentId) {
  await api(`content/${attachmentId}`, { method: 'DELETE', version: 'v1' })
  return { id: attachmentId, deleted: true }
}

export async function uploadOrUpdateAttachment(pageId, filePath, comment = 'draw.io diagram') {
  const fileName = basename(filePath)
  const existing = (await listAttachments(pageId)).find((a) => a.title === fileName)
  if (existing) {
    return updateAttachment(pageId, existing.id, filePath, comment)
  }
  return uploadAttachment(pageId, filePath, comment)
}
