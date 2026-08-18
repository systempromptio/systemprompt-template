/**
 * Doc-type registry — the ONE gate the pipeline goes through to reach a typed
 * FSD/ISD facade, mirroring the diagrams skill's type registry. The agnostic
 * core (doc/model, doc/md-to-storage, attachments) never imports a concrete type;
 * it only touches this registry and the uniform DocType interface.
 *
 * To add a doc type: create doc/types/<type>.mjs exporting a makeDocType(...)
 * facade and register it in TYPES here — nothing else changes.
 */

import { fsdType } from './fsd.mjs'
import { isdType } from './isd.mjs'

const TYPES = {
  fsd: fsdType,
  isd: isdType,
}

/** Resolve a doc type (default 'fsd' for undefined/unknown, matching the old canonFor). */
export function getDocType(type) {
  return TYPES[String(type || 'fsd').toLowerCase()] || TYPES.fsd
}

/** Registered doc-type names. */
export function listDocTypes() {
  return Object.keys(TYPES)
}
