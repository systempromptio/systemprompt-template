/**
 * `type: flow` registry entry — the DiagramType facade for flow / block diagrams.
 *
 * Responsibility: bind the flow validate/layout/emit into the uniform type interface
 *   ({@link import('../../lib/types.mjs').DiagramType}) the registry consumes.
 * Edit here when: you wire a new capability of this type into the interface.
 * Do NOT: add logic here — this is a thin facade; behavior lives in validate/layout/emit.
 */
import { validate, warnings } from './validate.mjs'
import { layout } from './layout/index.mjs'
import { emit } from './emit.mjs'

/** @type {import('../../lib/types.mjs').DiagramType} */
export default {
  type: 'flow',
  title: 'Flow / block diagram (boxes + optional decisions on a manual grid)',
  validate,
  warnings,
  layout,
  emit,
}
