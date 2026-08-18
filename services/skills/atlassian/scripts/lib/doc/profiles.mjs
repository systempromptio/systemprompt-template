/**
 * Doc-type profiles for the Confluence publisher.
 *
 * Each profile declares:
 *   parentEnv      – env var that holds the default Confluence parent page ID
 *   toc            – TOC macro config { min, max } (heading levels to include)
 *   badgeMap       – { 'status text (lowercase)': 'ConfluenceColour' }
 *                    maps backtick-wrapped inline text to Confluence status macros
 *   versionMessage – Confluence page version message on update
 *   contentAppearance – page width: 'default' (Narrow, fixed) or 'full-width'
 *
 * To add a new doc type (e.g. "spec"), add an entry here — nothing else changes.
 *
 * badgeMap is sourced from status-vocab.mjs (STATUS_COLOURS) so the inline badge
 * colours can never drift from the header-lozenge colours (they once did:
 * "on review" was Yellow here and Blue in the renderer).
 */

import { STATUS_COLOURS } from './status-vocab.mjs'

const PROFILES = {
  fsd: {
    parentEnv: 'CONFLUENCE_FSD_PARENT_ID',
    toc: { min: 2, max: 3 },
    badgeMap: STATUS_COLOURS,
    versionMessage: 'FSD update',
    contentAppearance: 'full-width',
  },
  isd: {
    parentEnv: 'CONFLUENCE_ISD_PARENT_ID',
    toc: { min: 2, max: 3 },
    badgeMap: STATUS_COLOURS,
    versionMessage: 'ISD update',
    contentAppearance: 'full-width',
  },
}

/**
 * Returns the profile for the given docType, or null if docType is absent/unknown.
 * Throws when docType is provided but not registered (fast-fail on typos).
 */
export function resolveProfile(docType) {
  if (!docType) return null
  const profile = PROFILES[docType.toLowerCase()]
  if (!profile) {
    const known = Object.keys(PROFILES).join(', ')
    throw new Error(`Unknown --type "${docType}". Known types: ${known}`)
  }
  return profile
}

/**
 * Build the Confluence TOC macro string for the given profile.
 */
export function tocMacro(profile) {
  const { min, max } = profile.toc
  return (
    `<ac:structured-macro ac:name="toc">` +
    `<ac:parameter ac:name="minLevel">${min}</ac:parameter>` +
    `<ac:parameter ac:name="maxLevel">${max}</ac:parameter>` +
    `<ac:parameter ac:name="type">list</ac:parameter>` +
    `</ac:structured-macro>`
  )
}
