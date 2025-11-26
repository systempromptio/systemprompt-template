/**
 * Dashboard list section component.
 *
 * Renders a list of items in a dashboard section with optional icons and status indicators.
 *
 * @module artifacts/renderers/dashboard/ListSection
 */

import React from 'react'
import { FileText, Eye, TrendingUp, Activity, AlertCircle, CheckCircle, XCircle } from 'lucide-react'

interface ListItem {
  text: string
  label?: string
  value?: string
  badge?: string
  secondary?: string
  icon?: string
  status?: 'success' | 'warning' | 'error' | 'info'
}

interface ListSectionProps {
  data: unknown
}

const iconMap: Record<string, React.ComponentType<{ className?: string }>> = {
  'file-text': FileText,
  'eye': Eye,
  'trending-up': TrendingUp,
  'activity': Activity,
  'alert-circle': AlertCircle,
  'check-circle': CheckCircle,
  'x-circle': XCircle,
}

const statusColors: Record<string, string> = {
  success: 'text-success',
  warning: 'text-warning',
  error: 'text-error',
  info: 'text-primary',
}

const statusBgColors: Record<string, string> = {
  success: 'bg-success/10 border-success/30',
  warning: 'bg-warning/10 border-warning/30',
  error: 'bg-error/10 border-error/30',
  info: 'bg-primary/10 border-primary/30',
}

export const ListSection = React.memo(function ListSection({
  data,
}: ListSectionProps) {
  const listData = data as { items: ListItem[] }

  if (!listData.items || !Array.isArray(listData.items)) {
    return <div className="text-secondary">Invalid list data</div>
  }

  return (
    <ul className="space-y-2">
      {listData.items.map((item, idx) => {
        const Icon = item.icon ? iconMap[item.icon] : null
        const statusClass = item.status ? statusColors[item.status] : 'text-primary'
        const statusBgClass = item.status ? statusBgColors[item.status] : ''
        const isRichFormat = item.label || item.value || item.badge || item.secondary

        if (isRichFormat) {
          return (
            <li key={idx} className={`flex items-start gap-3 p-3 rounded-lg border ${statusBgClass || 'border-primary-10 bg-surface'}`}>
              {Icon && <Icon className={`w-5 h-5 flex-shrink-0 mt-0.5 ${statusClass}`} />}
              <div className="flex-1 min-w-0">
                <div className="flex items-baseline justify-between gap-2 flex-wrap">
                  <span className="font-medium text-primary">{item.label || item.text}</span>
                  {item.badge && (
                    <span className="text-xs px-2 py-0.5 rounded bg-primary/10 text-secondary font-medium">
                      {item.badge}
                    </span>
                  )}
                </div>
                {item.value && (
                  <div className="text-sm text-secondary mt-0.5">{item.value}</div>
                )}
                {item.secondary && (
                  <div className="text-xs text-secondary mt-1">{item.secondary}</div>
                )}
              </div>
            </li>
          )
        }

        return (
          <li key={idx} className="flex items-start gap-3 p-2 rounded hover:bg-surface-variant">
            {Icon && <Icon className={`w-5 h-5 flex-shrink-0 mt-0.5 ${statusClass}`} />}
            <span className="text-primary">{item.text}</span>
          </li>
        )
      })}
    </ul>
  )
})
