/**
 * Dashboard table section component.
 *
 * Renders table data in a dashboard section.
 *
 * @module artifacts/renderers/dashboard/TableSection
 */

import React from 'react'
import type { Artifact } from '@/types/artifact'
import type { ColumnType } from '@/types/artifacts'
import { TableRenderer } from '../TableRenderer'

interface TableSectionProps {
  data: unknown
  artifact: Artifact
}

export const TableSection = React.memo(function TableSection({
  data,
  artifact,
}: TableSectionProps) {
  const tableData = data as {
    columns: Array<{ name: string; label?: string; column_type?: string }>
    items: Array<Record<string, unknown>>
    hints?: {
      filterable?: boolean
      sortable_columns?: string[]
      page_size?: number
    }
  }

  if (!tableData.columns || !tableData.items) {
    return <div className="text-secondary">Invalid table data</div>
  }

  const mockArtifact: Artifact = {
    ...artifact,
    parts: [{
      kind: 'data',
      data: { items: tableData.items },
    }],
  }

  const columnNames = tableData.columns.map(c => c.name)
  const columnTypes: Record<string, ColumnType> = {}
  tableData.columns.forEach(col => {
    if (col.column_type) {
      columnTypes[col.name] = col.column_type as ColumnType
    }
  })

  // Extract hints from data if available, otherwise use defaults
  const filterable = tableData.hints?.filterable ?? false
  const sortableColumns = tableData.hints?.sortable_columns ?? columnNames
  const pageSize = tableData.hints?.page_size ?? 10

  return (
    <div className="border border-primary-10 rounded-lg overflow-hidden bg-surface">
      <TableRenderer
        artifact={mockArtifact}
        hints={{
          columns: columnNames,
          sortable_columns: sortableColumns,
          filterable: filterable,
          page_size: pageSize,
          column_types: columnTypes,
        }}
      />
    </div>
  )
})
