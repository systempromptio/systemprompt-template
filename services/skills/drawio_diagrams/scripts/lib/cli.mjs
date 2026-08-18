/**
 * Shared CLI helpers for the command entry points (generate/validate/render).
 *
 * Responsibility: one small, well-tested arg parser + exit helpers so the three CLIs behave
 *   consistently and don't each hand-roll their own parsing.
 * Inputs/Outputs: argv in, { flags, positional } out; exit helpers terminate the process.
 * Edit here when: you change how flags are parsed or how CLI errors are reported.
 * Do NOT: put diagram logic here — this is I/O plumbing only.
 */

/**
 * Parse argv into flags + positionals. Supports both `--key value` and `--key=value`; flags
 * named in `booleans` take no value. A `--key` with no following value (or followed by another
 * `--flag`) becomes `true`.
 * @param {string[]} argv
 * @param {{ booleans?: string[] }} [opts]
 * @returns {{ flags: Record<string, string|boolean>, positional: string[] }}
 */
export function parseArgs(argv, { booleans = [] } = {}) {
  const flags = {}
  const positional = []
  for (let i = 0; i < argv.length; i++) {
    const a = argv[i]
    if (!a.startsWith('--')) {
      positional.push(a)
      continue
    }
    const eq = a.indexOf('=')
    if (eq !== -1) {
      flags[a.slice(2, eq)] = a.slice(eq + 1)
      continue
    }
    const key = a.slice(2)
    if (booleans.includes(key)) {
      flags[key] = true
      continue
    }
    const next = argv[i + 1]
    if (next != null && !next.startsWith('--')) {
      flags[key] = next
      i++
    } else {
      flags[key] = true
    }
  }
  return { flags, positional }
}

/** Print a JSON failure envelope to stderr and exit (default code 1). Used by generate. */
export function failJson(error, code = 1) {
  console.error(JSON.stringify({ ok: false, error }, null, 2))
  process.exit(code)
}

/** Print a plain message to stderr and exit (default code 2 = usage error). */
export function die(message, code = 2) {
  console.error(message)
  process.exit(code)
}
