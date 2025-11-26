/**
 * Dashboard status section component.
 *
 * Renders status indicators showing system or data status.
 *
 * @module artifacts/renderers/dashboard/StatusSection
 */

import React from 'react'
import { CheckCircle, AlertCircle, XCircle, Activity } from 'lucide-react'

interface StatusIndicator {
  label: string
  status: 'success' | 'warning' | 'error' | 'info'
  message?: string
}

interface StatusSectionProps {
  data: unknown
}

const statusConfig: Record<string, { icon: React.ComponentType<{ className?: string }>; color: string; bg: string; border: string }> = {
  success: { icon: CheckCircle, color: 'text-success', bg: 'bg-success/10', border: 'border-success/30' },
  warning: { icon: AlertCircle, color: 'text-warning', bg: 'bg-warning/10', border: 'border-warning/30' },
  error: { icon: XCircle, color: 'text-error', bg: 'bg-error/10', border: 'border-error/30' },
  info: { icon: Activity, color: 'text-primary', bg: 'bg-primary/10', border: 'border-primary/30' },
}

export const StatusSection = React.memo(function StatusSection({
  data,
}: StatusSectionProps) {
  const statusData = data as { indicators: StatusIndicator[] }

  if (!statusData.indicators || !Array.isArray(statusData.indicators)) {
    return <div className="text-secondary">Invalid status data</div>
  }

  return (
    <div className="space-y-3">
      {statusData.indicators.map((indicator, idx) => {
        const config = statusConfig[indicator.status]
        const Icon = config.icon

        return (
          <div
            key={idx}
            className={`flex items-start gap-3 p-3 border rounded-lg ${config.bg} ${config.border}`}
          >
            <Icon className={`w-5 h-5 flex-shrink-0 ${config.color}`} />
            <div className="flex-1">
              <div className={`font-medium ${config.color}`}>{indicator.label}</div>
              {indicator.message && (
                <div className="text-sm text-secondary mt-1">{indicator.message}</div>
              )}
            </div>
          </div>
        )
      })}
    </div>
  )
})
