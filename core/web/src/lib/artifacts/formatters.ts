export const formatCurrency = (value: number): string => {
  return new Intl.NumberFormat('en-US', {
    style: 'currency',
    currency: 'USD',
  }).format(value)
}

export const formatPercentage = (value: number): string => {
  return `${value.toFixed(1)}%`
}

export const formatInteger = (value: number): string => {
  return new Intl.NumberFormat('en-US').format(Math.floor(value))
}

export const formatDatetime = (value: string | number): string => {
  try {
    const date = typeof value === 'string' ? new Date(value) : new Date(value)
    if (isNaN(date.getTime())) {
      return String(value)
    }
    return new Intl.DateTimeFormat('en-US', {
      month: 'short',
      day: 'numeric',
      year: 'numeric',
      hour: 'numeric',
      minute: '2-digit',
      hour12: true,
    }).format(date)
  } catch {
    return String(value)
  }
}

export interface BadgeStyle {
  text: string
  color: string
}

export const formatBadge = (value: string): BadgeStyle => {
  const colorMap: Record<string, string> = {
    admin: 'blue',
    user: 'green',
    guest: 'gray',
    active: 'green',
    inactive: 'yellow',
    suspended: 'red',
    error: 'red',
    warning: 'yellow',
    success: 'green',
    healthy: 'green',
    critical: 'red',
  }

  return {
    text: value,
    color: colorMap[value.toLowerCase()] || 'gray',
  }
}

/**
 * Get badge color for a value.
 * Used by table cell formatter to determine badge styling.
 */
export function getBadgeColor(value: string): string {
  const colorMap: Record<string, string> = {
    admin: 'blue',
    user: 'green',
    guest: 'gray',
    active: 'green',
    inactive: 'yellow',
    suspended: 'red',
    error: 'red',
    warning: 'yellow',
    success: 'green',
    healthy: 'green',
    critical: 'red',
  }
  return colorMap[value.toLowerCase()] || 'gray'
}
