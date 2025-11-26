export interface AvatarColors {
  primary: string
  secondary: string
  primaryRgb: string
  secondaryRgb: string
}

export function getInitials(username?: string | null, email?: string | null): string {
  if (username) {
    const trimmed = username.trim()
    const words = trimmed.split(/\s+/)

    if (words.length >= 2) {
      return (words[0][0] + words[words.length - 1][0]).toUpperCase()
    }

    return trimmed.slice(0, 2).toUpperCase()
  }

  if (email) {
    const localPart = email.split('@')[0]
    return localPart.slice(0, 2).toUpperCase()
  }

  return 'U'
}

function hashString(str: string): number {
  let hash = 0
  for (let i = 0; i < str.length; i++) {
    const char = str.charCodeAt(i)
    hash = ((hash << 5) - hash) + char
    hash = hash & hash
  }
  return Math.abs(hash)
}

function getDistinctiveHue(hash: number): number {
  const goldenRatioConjugate = 0.618033988749895
  const normalized = (hash * goldenRatioConjugate) % 1
  return Math.floor(normalized * 360)
}

function hslToRgb(h: number, s: number, l: number): string {
  s /= 100
  l /= 100

  const k = (n: number) => (n + h / 30) % 12
  const a = s * Math.min(l, 1 - l)
  const f = (n: number) => l - a * Math.max(-1, Math.min(k(n) - 3, Math.min(9 - k(n), 1)))

  const r = Math.round(255 * f(0))
  const g = Math.round(255 * f(8))
  const b = Math.round(255 * f(4))

  return `${r}, ${g}, ${b}`
}

export function generateColorPalette(userId?: string | null): AvatarColors {
  if (!userId) {
    return {
      primary: 'hsl(28, 91%, 60%)',
      secondary: 'hsl(38, 91%, 55%)',
      primaryRgb: '246, 147, 60',
      secondaryRgb: '246, 194, 60',
    }
  }

  const hash = hashString(userId)

  const hue1 = getDistinctiveHue(hash)
  const hue2 = (hue1 + 137.5) % 360

  const satVariation = (hash >> 8) % 25
  const lightVariation = (hash >> 16) % 20

  const saturation = 65 + satVariation
  const lightness = 50 + lightVariation

  const primaryHsl = `hsl(${hue1}, ${saturation}%, ${lightness}%)`
  const secondaryHsl = `hsl(${hue2}, ${saturation - 5}%, ${lightness - 8}%)`

  return {
    primary: primaryHsl,
    secondary: secondaryHsl,
    primaryRgb: hslToRgb(hue1, saturation, lightness),
    secondaryRgb: hslToRgb(hue2, saturation - 5, lightness - 8),
  }
}

export function getContrastColor(hsl: string): string {
  const match = hsl.match(/hsl\((\d+),\s*(\d+)%,\s*(\d+)%\)/)
  if (!match) return '#FFFFFF'

  const lightness = parseInt(match[3])

  return lightness > 50 ? '#000000' : '#FFFFFF'
}
