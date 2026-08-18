/**
 * `type: sequence` registry entry — the DiagramType facade for sequence diagrams.
 *
 * Responsibility: bind the sequence validate/layout/emit into the uniform type interface
 *   ({@link import('../../lib/types.mjs').DiagramType}) the registry consumes.
 * Edit here when: you wire a new capability of this type into the interface (e.g. add a
 *   `parse` for reverse .drawio -> spec, currently intentionally omitted).
 * Do NOT: add logic here — this is a thin facade; behavior lives in validate/layout/emit.
 */
import { validate, warnings } from './validate/index.mjs'
import { layout } from './layout/index.mjs'
import { emit } from './emit.mjs'

/** @type {import('../../lib/types.mjs').DiagramType} */
export default {
  type: 'sequence',
  title: 'UML sequence diagram (participants, call/return/self messages, notes)',
  validate,
  warnings,
  layout,
  emit,
}
