/**
 * Type-specific validation for `type: sequence` — structural + authoring-budget checks.
 *
 * Responsibility: reject any spec that would render wrong or overflow the fixed layout, with
 *   a clear, self-explanatory message that names the offending item and cites its rule. This
 *   is the authoring safety net: an invalid spec produces NO files. Flow rules R1/R2 live in
 *   ./flow.mjs; the rule catalog + citation format live in ../rules.mjs.
 * Inputs: {@link import('../../../lib/types.mjs').Spec}. Outputs: string[] ([] = valid).
 * Edit here when: you add/adjust a structural or text-budget check (R3/R4/R5). For flow
 *   semantics edit ./flow.mjs; for rule ids/sections edit ../rules.mjs.
 * Do NOT: run the flow simulation on a structurally-broken message list (guarded below).
 */
import { L } from '../geometry.mjs'
import { cite } from '../rules.mjs'
import { validateFlow } from './flow.mjs'

/**
 * @param {import('../../../lib/types.mjs').Spec} spec
 * @returns {string[]}
 */
export function validate(spec) {
  const errors = []

  const participants = spec.participants
  if (!Array.isArray(participants) || participants.length === 0) {
    errors.push('participants: at least one participant is required')
  }

  const ids = new Set()
  for (const p of participants || []) {
    if (!p || typeof p !== 'object' || !p.id) {
      errors.push('participants: each participant needs an `id`')
      continue
    }
    if (ids.has(p.id)) errors.push(`participants: duplicate id "${p.id}"`)
    ids.add(p.id)
    if (p.kind && !['actor', 'box'].includes(p.kind)) {
      errors.push(`participant "${p.id}": kind must be "actor" or "box" (got "${p.kind}")`)
    }
    if ('label' in p) {
      errors.push(
        `participant "${p.id}": "label" was renamed to "title" (with an optional "subtitle"). ${cite('R3')}`,
      )
    }
    // R3 text budgets (title/subtitle length) are SOFT — reported by `warnings()`, not here.
  }

  const messages = spec.messages
  if (!Array.isArray(messages) || messages.length === 0) {
    errors.push('messages: at least one message is required')
  }
  const msgErrorsBefore = errors.length
  ;(messages || []).forEach((m, i) => {
    if (!m || typeof m !== 'object') {
      errors.push(`messages[${i}]: must be an object`)
      return
    }
    if (!['call', 'return', 'self'].includes(m.kind)) {
      errors.push(`messages[${i}]: kind must be one of call|return|self (got "${m.kind}")`)
    }
    if (!m.from || !ids.has(m.from)) {
      errors.push(`messages[${i}]: unknown "from" participant "${m.from}"`)
    }
    if (m.kind === 'self') {
      if (m.to && m.to !== m.from) {
        errors.push(`messages[${i}]: self message "to" must equal "from"`)
      }
    } else if (!m.to || !ids.has(m.to)) {
      errors.push(`messages[${i}]: unknown "to" participant "${m.to}"`)
    }
  })
  ;(spec.notes || []).forEach((note, i) => {
    if (!note || typeof note !== 'object') {
      errors.push(`notes[${i}]: must be an object`)
      return
    }
    if (!note.text) {
      errors.push(`notes[${i}]: text is required`)
    }
    // R5 note-length budget is SOFT — reported by `warnings()`, not here.
    // A note now anchors under exactly ONE participant (was `over`/`near`).
    if ('over' in note || 'near' in note) {
      errors.push(
        `notes[${i}]: "over"/"near" are removed — a note is anchored under ONE participant via ` +
          `"under: <id>". ${cite('R5')}`,
      )
    }
    if (!note.under || !ids.has(note.under)) {
      errors.push(
        `notes[${i}]: "under" must reference a known participant id (a note sits under exactly one ` +
          `participant). ${cite('R5')}`,
      )
    }
  })

  // Flow rules (R1/R2) only make sense once the message list is structurally sound —
  // otherwise the stack simulation would report noise on top of the real problem.
  const messagesAreSound = Array.isArray(messages) && messages.length > 0 && errors.length === msgErrorsBefore
  if (messagesAreSound) {
    errors.push(...validateFlow(spec, participants))
  }

  // R6: left→right causality. A `call` hands control rightward (to a participant drawn to its
  // right); only a `return` travels right→left, back to the caller. This keeps the picture's
  // time-forward reading and matches the call-stack model (the caller always sits to the left).
  const indexOfId = new Map((participants || []).map((p, i) => [p && p.id, i]))
  const plabel = (id) => {
    const p = (participants || []).find((pp) => pp && pp.id === id)
    return p && p.title ? p.title : id
  }
  ;(messages || []).forEach((m, i) => {
    if (!m || typeof m !== 'object' || !ids.has(m.from)) return
    if (m.kind === 'call') {
      if (!ids.has(m.to)) return
      if (indexOfId.get(m.to) < indexOfId.get(m.from)) {
        errors.push(
          `messages[${i}] (call ${plabel(m.from)} → ${plabel(m.to)}): a call cannot go right-to-left. Calls flow ` +
            `left→right (control is handed to a participant drawn to the right); only a return travels leftward, ` +
            `back to the caller. Reorder the participants or model this as a return. ${cite('R6')}`,
        )
      }
    } else if (m.kind === 'return') {
      if (!ids.has(m.to)) return
      if (indexOfId.get(m.to) > indexOfId.get(m.from)) {
        errors.push(
          `messages[${i}] (return ${plabel(m.from)} → ${plabel(m.to)}): a return must travel right-to-left, back to ` +
            `its caller (drawn to the left). A rightward hand-off is a call, not a return. ${cite('R6')}`,
        )
      }
    }
  })

  // R4 message-label budgets are SOFT — reported by `warnings()`, not here.

  return errors
}

/**
 * Soft text-budget advisories (R3 titles/subtitles, R4 message labels, R5 note text). These NEVER
 * block a file: header/box widths are fixed constants, so over-budget text simply overflows — the
 * author is told, but a strong model can also read the render and trim.
 * @param {import('../../../lib/types.mjs').Spec} spec
 * @returns {string[]}
 */
export function warnings(spec) {
  const warns = []
  const participants = Array.isArray(spec?.participants) ? spec.participants : []
  for (const p of participants) {
    if (!p || typeof p !== 'object' || !p.id) continue
    const title = p.title ?? p.id
    if (String(title).length > L.TITLE_MAX_CHARS) {
      warns.push(
        `participant "${p.id}": title "${title}" is ${String(title).length} chars; the header box fits ~` +
          `${L.TITLE_MAX_CHARS} — it will overflow, so shorten it and move detail into "subtitle". ${cite('R3')}`,
      )
    }
    if (p.subtitle != null && String(p.subtitle).length > L.SUBTITLE_MAX_CHARS) {
      warns.push(
        `participant "${p.id}": subtitle "${p.subtitle}" is ${String(p.subtitle).length} chars; it fits ~` +
          `${L.SUBTITLE_MAX_CHARS} — trim it. ${cite('R3')}`,
      )
    }
  }
  const messages = Array.isArray(spec?.messages) ? spec.messages : []
  messages.forEach((m, i) => {
    if (!m || typeof m !== 'object' || m.text == null) return
    const len = String(m.text).length
    if (m.kind === 'self') {
      if (len > L.SELF_LABEL_MAX_CHARS) {
        warns.push(
          `messages[${i}] (self): label "${m.text}" is ${len} chars; a self label fits ~` +
            `${L.SELF_LABEL_MAX_CHARS} (wraps to a second line) — shorten it. ${cite('R4')}`,
        )
      }
    } else if (m.kind === 'call' || m.kind === 'return') {
      if (len > L.MSG_LABEL_MAX_CHARS) {
        warns.push(
          `messages[${i}] (${m.kind}): label "${m.text}" is ${len} chars; a message label is a single ` +
            `short line of ~${L.MSG_LABEL_MAX_CHARS} chars — use a terser verb phrase. ${cite('R4')}`,
        )
      }
    }
  })
  const notes = Array.isArray(spec?.notes) ? spec.notes : []
  notes.forEach((note, i) => {
    if (!note || typeof note !== 'object' || !note.text) return
    if (String(note.text).length > L.NOTE_MAX_CHARS) {
      warns.push(
        `notes[${i}]: text is ${String(note.text).length} chars; a note fits ~${L.NOTE_MAX_CHARS} ` +
          `(wraps within 1.5x a participant box) — shorten it. ${cite('R5')}`,
      )
    }
  })
  return warns
}
