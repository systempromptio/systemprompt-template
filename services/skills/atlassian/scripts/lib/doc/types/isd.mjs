/**
 * ISD (Integration Specification Document) type vocabulary.
 *
 * Only the ISD-specific strings live here; all behaviour comes from base.mjs.
 *
 * bodySections: the chrome is only the card + H3 approval groups + "## Reference
 * Materials"; everything from the first content H2 to EOF is body. Real ISDs
 * vary in which content section comes first — some lead with "Overview", the
 * canonical template leads with "Requirements" (the Area/Requirement/Details
 * decomposition), others go straight to "Integration Specification". We list all
 * of them so the header/body split lands right after the chrome regardless of
 * the variant; the FIRST one present triggers the body. If this list were too
 * narrow (e.g. only "Requirements"), a doc that omits it would run the split off
 * the end and mis-read its content tables as approval rosters.
 *
 * requiredH2 is the shape common to every ISD variant we accept: the card,
 * Reference Materials, and Integration Specification. "Requirements" and
 * "In Scope Functional Requirements" are intentionally NOT required — different
 * ISD generations carry one, the other, or both.
 *
 * requirementSectionTitle stays null: coded-requirement collection for the ISD
 * (the WBS-keyed blocks under "In Scope Functional Requirements", whose codes can
 * lead with a digit e.g. 3PI.CON.OTR) is a separate enhancement — the shared
 * REQ_HEADING_RE only matches letter-leading codes today.
 */

import { makeDocType } from './base.mjs'

export const isdType = makeDocType({
  type: 'isd',
  cardHeading: 'General ISD Information',
  bodySections: [
    'Overview',
    'Requirements',
    'Integration Specification',
    'In Scope Functional Requirements',
    'Deferred Requirements',
    'Change Requests',
  ],
  requiredH2: ['General ISD Information', 'Reference Materials', 'Integration Specification'],
  requirementSectionTitle: null,
})
