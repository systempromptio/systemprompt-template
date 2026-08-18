/**
 * Shared factory for the typed FSD/ISD document types. FSD and ISD share the
 * entire chrome skeleton (a header card, H3 approval groups, a Reference
 * Materials table) and differ ONLY in vocabulary: the card heading, the body
 * section names, the required H2s, and which body section (if any) holds coded
 * requirements. `makeDocType(config)` closes over that vocabulary and returns
 * the uniform DocType facade the registry (index.mjs) and the pipeline consume.
 *
 * Responsibility: the doc-type-SPECIFIC views (documentCard/approvals),
 *   validation, mention collection, and requirement parsing. All generic
 *   markdown parsing/serialization stays in doc/model.mjs; all status vocabulary
 *   in doc/status-vocab.mjs. This is the ONLY place FSD/ISD strings may live.
 * Edit here when: the shared typed behaviour changes. Add a config knob rather
 *   than branching on `type`.
 *
 * DocType facade:
 *   { type, cardHeading, bodySections, requiredH2, requirementSectionTitle,
 *     propertiesId, parse(md), validate(model), validateFormat(md),
 *     collectMentionNames(model), deriveCard(sections), deriveApprovals(sections),
 *     parseRequirements(body) }
 */

import { parseDoc, collectHeadings, stripMention } from '../model.mjs'
import { parseStorageToModel } from '../storage-to-doc.mjs'
import { statusValues, deriveClientName } from '../status-vocab.mjs'

// FA = Functional Analyst (this org's name for the BA role on an ISD roster).
const ROLE_HINTS = ['SA', 'FA', 'FE', 'BE', 'QA', 'PO', 'DEV', 'BA', 'PM']

// Requirement heading: one or more codes, then a whitespace-surrounded dash,
// then the title. Codes are upper-case tokens that may contain dots and hyphens
// (GH.NAV, RQ-701) and may be comma-grouped (RQ-701, RQ-710). The separator must
// have surrounding whitespace so an intra-code hyphen (RQ-701) is not mistaken
// for it. Capture 1 is the whole code list; parseRequirements splits it.
const REQ_HEADING_RE = /^([A-Z][A-Z0-9.-]*(?:\s*,\s*[A-Z][A-Z0-9.-]*)*)\s+[-–—]\s+(.+)$/

export function makeDocType({
  type,
  cardHeading,
  bodySections,
  requiredH2,
  requirementSectionTitle = null,
  propertiesId = `${type}-header`,
}) {
  // Typed header-card view from the first header section's [label, value] rows.
  const deriveCard = (sections) => {
    const card = { wbsFeatureName: '', projectName: '', package: '', authorOwner: { name: '' }, status: '' }
    for (const [label = '', value = ''] of sections[0]?.rows || []) {
      const key = label.toLowerCase()
      if (/wbs/.test(key) && /feature/.test(key)) card.wbsFeatureName = value
      else if (/project name/.test(key)) card.projectName = value
      else if (/package/.test(key)) card.package = value
      else if (/author|owner/.test(key)) card.authorOwner = { name: stripMention(value) }
      else if (/status/.test(key)) card.status = value.toLowerCase()
    }
    return card
  }

  // Typed approval view: every header section after the header card, read as a
  // [role, status, name] roster (the common H3 shape). Used only for validation
  // and @mention collection — rendering uses the generic sections directly.
  // Role leads because column 1 is the Content Properties key on the wiki.
  const deriveApprovals = (sections) =>
    (sections.slice(1) || []).map((s) => ({
      label: s.label,
      rows: s.rows
        .filter(([role = '']) => role.trim())
        .map(([role = '', status = '', name = '']) => ({
          role: role.trim(),
          status: status.toLowerCase().trim(),
          name: stripMention(name),
        })),
    }))

  // Coded requirements under this type's requirement section (null → none, e.g.
  // ISD does not currently collect requirement codes). Fence-aware.
  const parseRequirements = (body) => {
    if (!requirementSectionTitle) return []
    const gate = requirementSectionTitle.toLowerCase()
    const lines = String(body || '').split(/\r?\n/)
    const requirements = []
    let inFence = false
    let inSection = false

    for (let n = 0; n < lines.length; n++) {
      const line = lines[n]
      const t = line.trim()
      if (t.startsWith('```') || t.startsWith('~~~')) { inFence = !inFence; continue }
      if (inFence) continue

      const hm = /^(#{2,6})\s+(.*\S)\s*$/.exec(line)
      if (!hm) continue
      const level = hm[1].length
      const title = hm[2].trim()

      if (level === 2) inSection = title.toLowerCase() === gate
      else if (level === 3 && inSection) {
        const rm = REQ_HEADING_RE.exec(title)
        if (rm) {
          const codes = rm[1].split(',').map((c) => c.trim()).filter(Boolean)
          requirements.push({ code: rm[1].trim(), codes, title: rm[2].trim(), line: n })
        }
      }
    }
    return requirements
  }

  const parse = (md) => {
    const model = parseDoc(md, { bodySections })
    return {
      ...model,
      documentCard: deriveCard(model.header.sections),
      approvals: deriveApprovals(model.header.sections),
    }
  }

  // Reverse of the publish pipeline: a page's STORAGE → the generic model. Driven
  // by this type's `cardHeading` (the detection anchor); the heavy DOM parsing is
  // the doc-type-agnostic engine in doc/storage-to-doc.mjs. Throws NotDocTypeError
  // when the storage is not this type, so the export CLI can fall back per page.
  const parseStorage = (storageXhtml, opts = {}) =>
    parseStorageToModel(storageXhtml, { cardHeading, ...opts })

  const validate = (model) => {
    const errors = []
    const warnings = []

    if (!model?.title) errors.push('Page title (# heading) is required.')

    // The client's own review word is valid only as this document spells the
    // client — the name comes from the "<Client> Approval" group label, so a
    // drifted spelling in a status cell is caught here.
    const known = statusValues(deriveClientName(model))

    const card = model?.documentCard || {}
    if (!card.wbsFeatureName) warnings.push('Header card: "WBS-Feature Name" is empty.')
    if (!card.projectName) warnings.push('Header card: "Project Name" is empty.')
    if (!card.authorOwner?.name) warnings.push('Header card: "Author/Owner" is empty.')
    if (card.status && !known.includes(card.status)) {
      errors.push(`Header card status "${card.status}" is not one of: ${known.join(', ')}.`)
    }

    for (const g of model?.approvals || []) {
      g.rows.forEach((row, i) => {
        const where = `Approval "${g.label}" row ${i + 1}`
        if (!row.name) errors.push(`${where}: approver name is required.`)
        if (row.status && !known.includes(row.status)) {
          errors.push(`${where}: status "${row.status}" is not one of: ${known.join(', ')}.`)
        }
        if (row.role && !ROLE_HINTS.includes(row.role)) {
          warnings.push(`${where}: role "${row.role}" is unusual (hints: ${ROLE_HINTS.join(', ')}).`)
        }
      })
    }

    ;(model?.references || []).forEach((row, i) => {
      if (!row.material) errors.push(`References row ${i + 1}: material is required.`)
    })

    const requirements = parseRequirements(model?.body || '')
    const seen = new Set()
    for (const req of requirements) {
      for (const code of req.codes || [req.code]) {
        if (seen.has(code)) errors.push(`Duplicate requirement code "${code}" in body.`)
        seen.add(code)
      }
    }
    if (requirementSectionTitle &&
        new RegExp(`##\\s*${requirementSectionTitle}`, 'i').test(model?.body || '') &&
        requirements.length === 0) {
      warnings.push(`"${requirementSectionTitle}" has no <CODE> - Title requirements.`)
    }

    return { ok: errors.length === 0, errors, warnings }
  }

  /**
   * Validate the markdown *structure* canon: every base H2 in this type's
   * requiredH2 is present (others allowed). Layout (dividers / blank-line
   * spacing) is owned entirely by the renderer and is deliberately NOT checked.
   */
  const validateFormat = (md) => {
    if (typeof md !== 'string') throw new TypeError('validateFormat expects a string')
    const errors = []
    const h2Titles = new Set(
      collectHeadings(md).filter((h) => h.level === 2).map((h) => h.title.toLowerCase()),
    )
    for (const req of requiredH2) {
      if (!h2Titles.has(req.toLowerCase())) errors.push(`Missing required H2 section: "## ${req}".`)
    }
    return { ok: errors.length === 0, errors, warnings: [] }
  }

  // All distinct person names referenced by the chrome (author + approvers) so
  // the publisher can resolve them to Confluence account ids for @mentions.
  const collectMentionNames = (model) => {
    const names = new Set()
    if (model?.documentCard?.authorOwner?.name) names.add(model.documentCard.authorOwner.name)
    for (const g of model?.approvals || []) for (const r of g.rows) if (r.name) names.add(r.name)
    return [...names]
  }

  return {
    type,
    cardHeading,
    bodySections,
    requiredH2,
    requirementSectionTitle,
    propertiesId,
    parse,
    parseStorage,
    validate,
    validateFormat,
    collectMentionNames,
    deriveCard,
    deriveApprovals,
    parseRequirements,
  }
}
