/**
 * Dashboard chart section component.
 *
 * Renders chart data in a dashboard section.
 *
 * @module artifacts/renderers/dashboard/ChartSection
 */

import React from 'react'
import type { Artifact } from '@/types/artifact'
import { ChartRenderer } from '../ChartRenderer'

interface ChartSectionProps {
  data: unknown
  artifact: Artifact
}

export const ChartSection = React.memo(function ChartSection({
  data,
  artifact,
}: ChartSectionProps) {
  const chartData = data as { labels: string[]; datasets: Array<{ label: string; data: number[]; color?: string }> }

  if (!chartData.labels || !chartData.datasets) {
    return <div className="text-secondary">Invalid chart data</div>
  }

  const mockArtifact: Artifact = {
    ...artifact,
    parts: [{
      kind: 'data',
      data: chartData,
    }],
  }

  return (
    <div className="border border-primary-10 rounded-lg p-4 bg-surface">
      <ChartRenderer artifact={mockArtifact} hints={{ chart_type: 'line' }} />
    </div>
  )
})
