import { ChevronUp, ChevronDown, Search, AlertTriangle } from 'lucide-react'
import type { Artifact } from '@/types/artifact'
import type { TableHints } from '@/types/artifacts'
import { extractTableData, formatCurrency, formatPercentage, formatDatetime, formatInteger, formatBadge } from '@/lib/artifacts'
import { useTableData } from '@/hooks/useTableData'

interface TableRendererProps {
  artifact: Artifact
  hints: TableHints
}

interface BadgeProps {
  color: string
  children: React.ReactNode
}

function Badge({ color, children }: BadgeProps) {
  const colorClasses: Record<string, string> = {
    blue: 'bg-primary/20 text-primary border border-primary-15',
    green: 'bg-success/20 text-success border border-success/30',
    yellow: 'bg-warning/20 text-warning border border-warning/30',
    red: 'bg-error/20 text-error border border-error/30',
    gray: 'bg-surface-variant text-secondary border border-primary-10',
  }

  return (
    <span className={`inline-flex items-center px-2 py-0.5 rounded text-xs font-medium ${colorClasses[color] || colorClasses.gray}`}>
      {children}
    </span>
  )
}

export function TableRenderer({ artifact, hints }: TableRendererProps) {
  const result = extractTableData(artifact)
  const rows = result.data
  const errors = result.errors

  const { columns, sortColumn, sortOrder, searchText, setSearchText, currentPage, setCurrentPage, filteredData, paginatedData, totalPages, pageSize, handleSort } = useTableData({ rows, hints })

  const formatCell = (value: unknown, columnName: string) => {
    if (value === null || value === undefined) {
      return <span className="text-disabled italic">null</span>
    }

    const type = hints.column_types?.[columnName]

    switch (type) {
      case 'currency':
        return formatCurrency(Number(value))
      case 'percentage':
        return formatPercentage(Number(value))
      case 'datetime':
        return formatDatetime(String(value))
      case 'integer':
        return formatInteger(Number(value))
      case 'badge': {
        const badge = formatBadge(String(value))
        return <Badge color={badge.color}>{badge.text}</Badge>
      }
      case 'link':
        return (
          <a href={String(value)} target="_blank" rel="noopener noreferrer" className="text-primary hover:underline font-medium">
            View
          </a>
        )
      default:
        return String(value)
    }
  }

  if (errors && errors.length > 0) {
    return (
      <div className="border border-warning bg-surface-variant rounded-lg p-4">
        <div className="flex items-start gap-3">
          <AlertTriangle className="w-5 h-5 text-warning flex-shrink-0 mt-0.5" />
          <div className="flex-1">
            <h3 className="text-sm font-semibold text-primary mb-2">
              Data Validation Errors
            </h3>
            <ul className="text-sm text-secondary space-y-1">
              {errors.map((error, i) => (
                <li key={i}>• {error}</li>
              ))}
            </ul>
          </div>
        </div>
      </div>
    )
  }

  if (rows.length === 0) {
    return <div className="text-secondary text-center py-8">No data available</div>
  }

  return (
    <div className="space-y-4">
      {hints.filterable && (
        <div className="relative">
          <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 w-4 h-4 text-disabled" />
          <input
            type="text"
            placeholder="Search..."
            value={searchText}
            onChange={(e) => setSearchText(e.target.value)}
            className="w-full pl-10 pr-4 py-2 border border-primary-10 bg-surface rounded-lg focus:outline-none focus:ring-2 focus:ring-primary text-primary"
          />
        </div>
      )}

      <div className="overflow-x-auto border border-primary-10 rounded-lg">
        <table className="w-full text-sm">
          <thead className="bg-surface-variant border-b border-primary-10">
            <tr>
              {columns.map(col => {
                const isSortable = hints.sortable_columns?.includes(col)
                const isCurrentSort = sortColumn === col

                return (
                  <th
                    key={col}
                    className={`px-4 py-3 text-left font-medium text-primary ${
                      isSortable ? 'cursor-pointer hover:bg-surface-dark' : ''
                    }`}
                    onClick={() => isSortable && handleSort(col)}
                  >
                    <div className="flex items-center gap-2">
                      <span>{col.replace(/_/g, ' ')}</span>
                      {isSortable && (
                        <span className="text-disabled">
                          {isCurrentSort && sortOrder === 'asc' ? (
                            <ChevronUp className="w-4 h-4" />
                          ) : isCurrentSort && sortOrder === 'desc' ? (
                            <ChevronDown className="w-4 h-4" />
                          ) : (
                            <ChevronDown className="w-4 h-4 opacity-30" />
                          )}
                        </span>
                      )}
                    </div>
                  </th>
                )
              })}
            </tr>
          </thead>
          <tbody>
            {paginatedData.map((row, idx) => (
              <tr
                key={idx}
                className={idx % 2 === 0 ? 'bg-surface' : 'bg-surface-variant'}
              >
                {columns.map(col => (
                  <td key={col} className="px-4 py-3 border-t border-primary-10">
                    {formatCell((row as Record<string, unknown>)[col], col)}
                  </td>
                ))}
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      {totalPages > 1 && (
        <div className="flex items-center justify-between">
          <div className="text-sm text-secondary">
            Showing {((currentPage - 1) * pageSize) + 1} to {Math.min(currentPage * pageSize, filteredData.length)} of {filteredData.length} results
          </div>
          <div className="flex gap-2">
            <button
              onClick={() => setCurrentPage(p => Math.max(1, p - 1))}
              disabled={currentPage === 1}
              className="px-3 py-1 border border-primary-10 bg-surface rounded hover:bg-surface-variant disabled:opacity-50 disabled:cursor-not-allowed text-primary"
            >
              Previous
            </button>
            <span className="px-3 py-1 text-primary">
              Page {currentPage} of {totalPages}
            </span>
            <button
              onClick={() => setCurrentPage(p => Math.min(totalPages, p + 1))}
              disabled={currentPage === totalPages}
              className="px-3 py-1 border border-primary-10 bg-surface rounded hover:bg-surface-variant disabled:opacity-50 disabled:cursor-not-allowed text-primary"
            >
              Next
            </button>
          </div>
        </div>
      )}
    </div>
  )
}
