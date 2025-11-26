/**
 * Dashboard metrics cards section component.
 *
 * Renders a grid of metric cards displaying key values with icons and status indicators.
 *
 * @module artifacts/renderers/dashboard/MetricsCardsSection
 */

import React from 'react'
import { FileText, Eye, TrendingUp, Activity, AlertCircle, CheckCircle, XCircle } from 'lucide-react'

interface MetricsCard {
  title: string
  value: string | number
  subtitle?: string
  icon?: string
  status?: 'success' | 'warning' | 'error' | 'info'
}

interface MetricsCardsSectionProps {
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
  success: 'border-success/30 bg-success/5',
  warning: 'border-warning/30 bg-warning/5',
  error: 'border-error/30 bg-error/5',
  info: 'border-primary/30 bg-primary/5',
}

export const MetricsCardsSection = React.memo(function MetricsCardsSection({
  data,
}: MetricsCardsSectionProps) {
  const cardData = data as { cards: MetricsCard[] }
  if (!cardData.cards || !Array.isArray(cardData.cards)) {
    return <div className="text-secondary">Invalid metrics cards data</div>
  }

  return (
    <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
      {cardData.cards.map((card, idx) => {
        const Icon = card.icon ? iconMap[card.icon] : Activity
        const statusClass = card.status ? statusColors[card.status] : 'border-primary-10 bg-surface-variant'

        return (
          <div
            key={idx}
            className={`border rounded-lg p-4 ${statusClass}`}
          >
            <div className="flex items-start justify-between mb-2">
              <div className="text-sm font-medium text-secondary">{card.title}</div>
              {Icon && <Icon className="w-5 h-5 text-primary opacity-60" />}
            </div>
            <div className="text-2xl font-bold text-primary mb-1">{card.value}</div>
            {card.subtitle && (
              <div className="text-xs text-secondary">{card.subtitle}</div>
            )}
          </div>
        )
      })}
    </div>
  )
})
