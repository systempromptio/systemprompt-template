/**
 * Shared argv parser for every Atlassian CLI (confluence.mjs, publish.mjs, the
 * export/validate scripts). One parser so flags behave identically everywhere
 * instead of each entrypoint hand-rolling its own loop.
 *
 * Responsibility: turn argv into { flags, positional }. Nothing Confluence- or
 *   Jira-specific lives here — this is I/O plumbing only.
 * Edit here when: you change how flags are parsed. Do NOT add domain logic.
 *
 * Supports:
 *   --key value       → flags.key = 'value'
 *   --key=value       → flags.key = 'value'
 *   --key             → flags.key = true            (boolean; also when a listed
 *                       boolean is followed by a value it still stays a boolean)
 *   repeated --key    → flags.key = ['v1','v2', …]  (only for keys in `repeatable`;
 *                       always an array, even for a single occurrence)
 */

/**
 * @param {string[]} argv
 * @param {{ booleans?: string[], repeatable?: string[] }} [opts]
 * @returns {{ flags: Record<string, string|boolean|string[]>, positional: string[] }}
 */
export function parseArgs(argv, { booleans = [], repeatable = [] } = {}) {
  const flags = {}
  const positional = []
  const booleanSet = new Set(booleans)
  const repeatableSet = new Set(repeatable)

  const assign = (key, value) => {
    if (repeatableSet.has(key)) {
      if (!Array.isArray(flags[key])) flags[key] = flags[key] === undefined ? [] : [flags[key]]
      flags[key].push(value)
    } else {
      flags[key] = value
    }
  }

  for (let i = 0; i < argv.length; i++) {
    const a = argv[i]
    if (!a.startsWith('--')) {
      positional.push(a)
      continue
    }
    const body = a.slice(2)
    const eq = body.indexOf('=')
    if (eq !== -1) {
      assign(body.slice(0, eq), body.slice(eq + 1))
      continue
    }
    if (booleanSet.has(body)) {
      flags[body] = true
      continue
    }
    const next = argv[i + 1]
    if (next != null && !next.startsWith('--')) {
      assign(body, next)
      i++
    } else {
      flags[body] = true
    }
  }

  return { flags, positional }
}
