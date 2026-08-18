import { test } from 'node:test'
import assert from 'node:assert/strict'
import { readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { dirname, join } from 'node:path'

import { getDocType, listDocTypes } from '../lib/doc/types/index.mjs'

const HERE = dirname(fileURLToPath(import.meta.url))
const fixture = (name) => readFileSync(join(HERE, '__fixtures__', name), 'utf8')

test('registry lists fsd + isd and defaults unknown/undefined to fsd', () => {
  assert.deepEqual(listDocTypes().sort(), ['fsd', 'isd'])
  assert.equal(getDocType('fsd').type, 'fsd')
  assert.equal(getDocType('ISD').type, 'isd', 'lookup is case-insensitive')
  assert.equal(getDocType(undefined).type, 'fsd')
  assert.equal(getDocType('nope').type, 'fsd')
})

test('each type exposes the uniform DocType facade', () => {
  for (const type of listDocTypes()) {
    const dt = getDocType(type)
    for (const k of ['parse', 'validate', 'validateFormat', 'collectMentionNames', 'deriveCard', 'deriveApprovals', 'parseRequirements']) {
      assert.equal(typeof dt[k], 'function', `${type}.${k} is a function`)
    }
  }
})

test('deriveCard maps header rows to the fixed card keys', () => {
  const sections = [
    {
      label: 'General FSD Information',
      rows: [
        ['WBS-Feature Name', 'W1 - Login'],
        ['Project Name', 'Acme'],
        ['Package', 'Auth'],
        ['Author/Owner', '@Jane Doe'],
        ['Status', 'In Progress'],
      ],
    },
  ]
  const card = getDocType('fsd').deriveCard(sections)
  assert.equal(card.wbsFeatureName, 'W1 - Login')
  assert.equal(card.projectName, 'Acme')
  assert.equal(card.package, 'Auth')
  assert.equal(card.authorOwner.name, 'Jane Doe', '@ stripped')
  assert.equal(card.status, 'in progress', 'status lowercased')
})

test('deriveApprovals reads H3 groups as role/status/name rosters', () => {
  const sections = [
    { label: 'General FSD Information', rows: [] },
    {
      label: 'Delivery',
      rows: [
        ['FE', 'Approved', '@Jane Doe'],
        ['', '', ''],
      ],
    },
  ]
  const approvals = getDocType('fsd').deriveApprovals(sections)
  assert.equal(approvals.length, 1)
  assert.equal(approvals[0].label, 'Delivery')
  assert.deepEqual(approvals[0].rows, [{ role: 'FE', status: 'approved', name: 'Jane Doe' }])
})

test('fsd.parseRequirements collects codes; isd collects none (documented gap)', () => {
  const body = [
    '## In Scope Functional Requirements',
    '',
    '### RQ-701 - Login',
    '',
    '### GH.NAV, GH.FOOT - Chrome',
  ].join('\n')
  const fsdReqs = getDocType('fsd').parseRequirements(body)
  assert.deepEqual(fsdReqs.map((r) => r.code), ['RQ-701', 'GH.NAV, GH.FOOT'])
  assert.deepEqual(fsdReqs[1].codes, ['GH.NAV', 'GH.FOOT'])

  // ISD's requirement section is a different heading and its gate is null.
  const isdReqs = getDocType('isd').parseRequirements(body)
  assert.deepEqual(isdReqs, [])
})

test('validate flags an unknown status and a missing approver name', () => {
  const model = {
    title: 'T',
    documentCard: { wbsFeatureName: 'W', projectName: 'P', authorOwner: { name: 'A' }, status: 'banana' },
    approvals: [{ label: 'G', rows: [{ name: '', role: 'FE', status: 'approved' }] }],
    references: [{ material: 'X' }],
    body: '',
  }
  const v = getDocType('fsd').validate(model)
  assert.equal(v.ok, false)
  assert.ok(v.errors.some((e) => /status "banana"/.test(e)))
  assert.ok(v.errors.some((e) => /approver name is required/.test(e)))
})

test('validate detects a duplicate requirement code in the body', () => {
  const model = {
    title: 'T',
    documentCard: { authorOwner: { name: 'A' } },
    approvals: [],
    references: [],
    body: ['## In Scope Functional Requirements', '### RQ-1 - A', '### RQ-1 - B'].join('\n'),
  }
  const v = getDocType('fsd').validate(model)
  assert.equal(v.ok, false)
  assert.ok(v.errors.some((e) => /Duplicate requirement code "RQ-1"/.test(e)))
})

test('validateFormat requires the type base H2s and allows extras', () => {
  const fsd = getDocType('fsd')
  const bad = fsd.validateFormat('# T\n\n## General FSD Information\n')
  assert.equal(bad.ok, false)
  assert.ok(bad.errors.some((e) => /Reference Materials/.test(e)))

  const good = fsd.validateFormat(fixture('example.fsd.md'))
  assert.equal(good.ok, true, good.errors.join('; '))
})

test('collectMentionNames gathers author + approvers, de-duplicated', () => {
  const model = {
    documentCard: { authorOwner: { name: 'Jane Doe' } },
    approvals: [
      { label: 'A', rows: [{ name: 'Jane Doe' }, { name: 'John Roe' }] },
      { label: 'B', rows: [{ name: 'John Roe' }] },
    ],
  }
  assert.deepEqual(getDocType('fsd').collectMentionNames(model).sort(), ['Jane Doe', 'John Roe'])
})

test('fixtures validate clean for both types', () => {
  for (const type of ['fsd', 'isd']) {
    const dt = getDocType(type)
    const md = fixture(`example.${type}.md`)
    assert.equal(dt.validateFormat(md).ok, true, `${type} validateFormat`)
    assert.equal(dt.validate(dt.parse(md)).ok, true, `${type} validate`)
  }
})
