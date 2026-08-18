/**
 * Sequence flow rules R1 + R2 — the synchronous call-stack simulation.
 *
 * Responsibility: replay the messages as a call stack and enforce that only the participant
 *   on top may act (R1) and that every open frame is eventually closed by a return (R2). The
 *   stack holds one frame per open call `{ participant, caller, index }`.
 * Inputs: (spec, participants). Outputs: string[] of errors ([] = valid).
 * Edit here when: you refine what "the active flow" may do, or how unclosed frames are reported.
 * Do NOT: run this before the message list is structurally sound — the caller guards that, so
 *   the simulation never reports noise on top of a real structural error.
 */
import { cite } from '../rules.mjs'

/**
 * @param {import('../../../lib/types.mjs').Spec} spec
 * @param {import('../../../lib/types.mjs').Participant[]} participants
 * @returns {string[]}
 */
export function validateFlow(spec, participants) {
  const errors = []
  const label = (id) => {
    const p = (participants || []).find((pp) => pp && pp.id === id)
    return p && p.title ? `${p.title}` : id
  }
  const stack = []
  const active = () => (stack.length ? stack[stack.length - 1].participant : null)

  for (let i = 0; i < spec.messages.length; i++) {
    const m = spec.messages[i]
    const blocked = stack.length > 0 && m.from !== active()

    if (m.kind === 'call') {
      if (blocked) {
        errors.push(
          `messages[${i}] (call ${label(m.from)} → ${label(m.to)}): "${label(m.from)}" cannot start a call while ` +
            `"${label(active())}" is still executing. A flow may have only one outgoing call in flight — it must ` +
            `receive the return for its current call before initiating another. ${cite('R1')}`,
        )
      }
      stack.push({ participant: m.to, caller: m.from, index: i })
    } else if (m.kind === 'self') {
      if (stack.length === 0) {
        errors.push(
          `messages[${i}] (self ${label(m.from)}): no flow is active, so "${label(m.from)}" cannot do internal work ` +
            `here. Internal work must happen inside an open call. ${cite('R1')}`,
        )
      } else if (blocked) {
        errors.push(
          `messages[${i}] (self ${label(m.from)}): "${label(m.from)}" is not the active flow — "${label(active())}" is ` +
            `currently executing. Only the participant on top of the call stack may act. ${cite('R1')}`,
        )
      }
    } else if (m.kind === 'return') {
      if (stack.length === 0) {
        errors.push(
          `messages[${i}] (return ${label(m.from)} → ${label(m.to)}): there is no open call to return from. ` +
            `A return must close a call that is currently in flight. ${cite('R2')}`,
        )
      } else if (blocked) {
        errors.push(
          `messages[${i}] (return ${label(m.from)} → ${label(m.to)}): "${label(m.from)}" is not the active flow — ` +
            `"${label(active())}" is currently executing and must return first. ${cite('R1')}`,
        )
      } else {
        stack.pop()
      }
    }
  }

  // R2: anything still open never returned.
  for (const frame of stack) {
    errors.push(
      `messages[${frame.index}] (call ${label(frame.caller)} → ${label(frame.participant)}): "${label(frame.participant)}" ` +
        `never returns. Every call must be closed by a return to its caller as soon as the callee has nothing more to ` +
        `call — fire-and-forget is not allowed. ${cite('R2')}`,
    )
  }

  return errors
}
