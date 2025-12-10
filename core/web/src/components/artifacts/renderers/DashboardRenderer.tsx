/**
 * Dashboard artifact renderer component.
 *
 * Renders dashboard artifacts with support for multiple section types
 * (metrics, charts, tables, lists, status indicators).
 *
 * @module artifacts/renderers/DashboardRenderer
 */

import type { Artifact, DashboardHints, DashboardSection } from '@/types/artifact'
import { extractDashboardData, unwrapExtraction } from '@/lib/artifacts'
import { MetricsCardsSection } from './dashboard/MetricsCardsSection'
import { ChartSection } from './dashboard/ChartSection'
import { TableSection } from './dashboard/TableSection'
import { ListSection } from './dashboard/ListSection'
import { StatusSection } from './dashboard/StatusSection'

interface DashboardRendererProps {
  artifact: Artifact
  hints: DashboardHints
}

export function DashboardRenderer({ artifact, hints }: DashboardRendererProps) {
  const dashboardDataResult = extractDashboardData(artifact)
  const dashboardData = unwrapExtraction(dashboardDataResult)

  if (!dashboardData) {
    return <div className="text-secondary text-center py-8">Invalid dashboard data</div>
  }

  const layout = hints.layout || 'vertical'

  const sortedSections = [...dashboardData.sections].sort((a, b) => {
    const orderA = a.layout?.order ?? 999
    const orderB = b.layout?.order ?? 999
    return orderA - orderB
  })

  return (
    <div className="h-full flex flex-col">
      {dashboardData.title && (
        <div className="border-b border-primary-10 pb-4">
          <h2 className="text-2xl font-bold text-primary">{dashboardData.title}</h2>
          {dashboardData.description && (
            <p className="text-secondary mt-2">{dashboardData.description}</p>
          )}
        </div>
      )}

      <div className={`flex-1 overflow-y-auto space-y-6 ${layout === 'grid' ? 'grid grid-cols-1 md:grid-cols-2 gap-6' : ''}`}>
        {sortedSections.map((section) => (
          <DashboardSectionRenderer
            key={section.section_id}
            section={section}
            artifact={artifact}
          />
        ))}
      </div>
    </div>
  )
}

interface DashboardSectionRendererProps {
  section: DashboardSection
  artifact: Artifact
}

function DashboardSectionRenderer({ section, artifact }: DashboardSectionRendererProps) {
  return (
    <div className="space-y-3">
      {section.title && (
        <h3 className="text-lg font-semibold text-primary">{section.title}</h3>
      )}
      {section.section_type === 'metrics_cards' && <MetricsCardsSection data={section.data} />}
      {section.section_type === 'chart' && <ChartSection data={section.data} artifact={artifact} />}
      {section.section_type === 'table' && <TableSection data={section.data} artifact={artifact} />}
      {section.section_type === 'list' && <ListSection data={section.data} />}
      {section.section_type === 'status' && <StatusSection data={section.data} />}
      {!['metrics_cards', 'chart', 'table', 'list', 'status'].includes(section.section_type) && (
        <div className="text-secondary">Unknown section type: {section.section_type}</div>
      )}
    </div>
  )
}
