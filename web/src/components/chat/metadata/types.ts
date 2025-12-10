export interface TokenInfo {
  readonly input: number
  readonly output: number
  readonly formatted: string
}

export function formatTokenInfo(input?: number, output?: number): TokenInfo | null {
  if (!input && !output) return null

  const parts: string[] = []
  if (input) parts.push(`${input.toLocaleString()} in`)
  if (output) parts.push(`${output.toLocaleString()} out`)

  return {
    input: input ?? 0,
    output: output ?? 0,
    formatted: parts.join(' / '),
  }
}
