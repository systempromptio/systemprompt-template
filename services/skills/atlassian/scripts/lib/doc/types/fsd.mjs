/**
 * FSD (Functional Specification Document) type vocabulary.
 *
 * Only the FSD-specific strings live here; all behaviour comes from base.mjs.
 * The body starts at "In Scope Functional Requirements"; requirement codes
 * (GH.NAV, RQ-701, …) are collected from that section's H3 headings.
 */

import { makeDocType } from './base.mjs'

export const fsdType = makeDocType({
  type: 'fsd',
  cardHeading: 'General FSD Information',
  bodySections: ['In Scope Functional Requirements', 'Deferred Requirements', 'Change Requests'],
  requiredH2: [
    'General FSD Information',
    'Reference Materials',
    'In Scope Functional Requirements',
    'Deferred Requirements',
    'Change Requests',
  ],
  requirementSectionTitle: 'In Scope Functional Requirements',
})
