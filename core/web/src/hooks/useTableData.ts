/**
 * Hook for table data management.
 *
 * Handles sorting, filtering, and pagination logic.
 *
 * @module hooks/useTableData
 */

import { useState, useMemo } from 'react'
import type { TableHints } from '@/types/artifact'

interface UseTableDataProps {
  rows: unknown[]
  hints: TableHints
}

export function useTableData({ rows, hints }: UseTableDataProps) {
  const [sortColumn, setSortColumn] = useState<string | null>(hints.default_sort?.column || null)
  const [sortOrder, setSortOrder] = useState<'asc' | 'desc'>(hints.default_sort?.order || 'asc')
  const [searchText, setSearchText] = useState('')
  const [currentPage, setCurrentPage] = useState(1)

  const columns = hints.columns || (rows[0] && typeof rows[0] === 'object' ? Object.keys(rows[0] as object) : [])
  const pageSize = hints.page_size || 25

  const sortedData = useMemo(() => {
    if (!sortColumn) return rows

    return [...rows].sort((a, b) => {
      const aVal = (a as Record<string, unknown>)[sortColumn]
      const bVal = (b as Record<string, unknown>)[sortColumn]

      let comparison = 0
      if (aVal === null || aVal === undefined) comparison = 1
      else if (bVal === null || bVal === undefined) comparison = -1
      else if (typeof aVal === 'number' && typeof bVal === 'number') {
        comparison = aVal - bVal
      } else {
        comparison = String(aVal).localeCompare(String(bVal))
      }

      return sortOrder === 'asc' ? comparison : -comparison
    })
  }, [rows, sortColumn, sortOrder])

  const filteredData = useMemo(() => {
    if (!searchText || !hints.filterable) return sortedData

    return sortedData.filter(row => {
      if (typeof row !== 'object' || row === null) return false
      return Object.values(row as object).some(val => String(val).toLowerCase().includes(searchText.toLowerCase()))
    })
  }, [sortedData, searchText, hints.filterable])

  const totalPages = Math.ceil(filteredData.length / pageSize)
  const paginatedData = filteredData.slice((currentPage - 1) * pageSize, currentPage * pageSize)

  const handleSort = (column: string) => {
    if (!hints.sortable_columns?.includes(column)) return

    if (sortColumn === column) {
      setSortOrder(sortOrder === 'asc' ? 'desc' : 'asc')
    } else {
      setSortColumn(column)
      setSortOrder('asc')
    }
  }

  return {
    columns,
    sortColumn,
    sortOrder,
    searchText,
    setSearchText,
    currentPage,
    setCurrentPage,
    filteredData,
    paginatedData,
    totalPages,
    pageSize,
    handleSort,
  }
}
