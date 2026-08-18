/**
 * The single resolved path to the diagrams skill's `reverse.mjs` — the decoder
 * that turns a `.drawio` (embedded `data-spec`) back into the authored
 * ```drawio:<type>:<id> block. Shared so the confluence CLI, the bulk exporter,
 * and the tests all point at ONE location instead of hand-resolving it three
 * times with slightly different relative depths.
 */

import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

// This module lives at scripts/lib/diagrams/; the diagrams skill is a sibling of
// the atlassian skill under skills/plan/drawio-diagrams/.
export const REVERSE_CLI = resolve(
  dirname(fileURLToPath(import.meta.url)),
  '../../../../../plan/drawio-diagrams/scripts/reverse.mjs',
)
