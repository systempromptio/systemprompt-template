/**
 * Layout phase 1 — build the activation (call) tree from the message list.
 *
 * Responsibility: turn the ordered messages into a tree of activation frames via a call
 *   stack — `call X->Y` opens a frame on Y (child of X's open frame or the root), `return
 *   Y->X` closes Y's topmost open frame, `self X` is an internal action of X's frame. This
 *   tree (not arrow positions) is what the sizing phase measures.
 * Inputs: Message[]. Outputs: { root, allFrames }.
 * Edit here when: you change how messages map onto nested activations.
 * Do NOT: compute any Y/heights here — that is the activations phase.
 *
 * @typedef {Object} Frame
 * @property {string|null} participantId
 * @property {Array<{type:'call',frame:Frame,index:number}|{type:'self',index:number}|{type:'marker',index:number}>} actions
 * @property {number} openIndex
 * @property {number} closeIndex
 * @property {boolean} closed
 * @property {number} [top]
 * @property {number} [bottom]
 */

/**
 * @param {import('../../../lib/types.mjs').Message[]} messages
 * @returns {{ root: Frame, allFrames: Frame[] }}
 */
export function buildCallTree(messages) {
  const root = { participantId: null, actions: [], openIndex: -1, closeIndex: -1, closed: true }
  const allFrames = []
  const stack = [] // open frames, LIFO

  const openFrameOf = (id) => {
    for (let k = stack.length - 1; k >= 0; k--) if (stack[k].participantId === id) return stack[k]
    return null
  }
  const containerOf = (id) => openFrameOf(id) ?? root

  messages.forEach((m, i) => {
    if (m.kind === 'call') {
      const child = { participantId: m.to, actions: [], openIndex: i, closeIndex: -1, closed: false }
      containerOf(m.from).actions.push({ type: 'call', frame: child, index: i })
      allFrames.push(child)
      stack.push(child)
    } else if (m.kind === 'return') {
      const frame = openFrameOf(m.from)
      if (frame) {
        frame.closeIndex = i
        frame.closed = true
        stack.splice(stack.lastIndexOf(frame), 1)
      } else {
        // return with nothing open on `from`: keep it as a plain row.
        root.actions.push({ type: 'marker', index: i })
      }
    } else if (m.kind === 'self') {
      containerOf(m.from).actions.push({ type: 'self', index: i })
    }
  })

  return { root, allFrames }
}
