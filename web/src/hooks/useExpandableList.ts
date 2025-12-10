import { useState, useCallback } from 'react'

interface UseExpandableListReturn {
  readonly expandedIds: ReadonlySet<string>
  isExpanded: (id: string) => boolean
  toggle: (id: string) => void
  expand: (id: string) => void
  collapse: (id: string) => void
  collapseAll: () => void
  readonly count: number
}

export function useExpandableList(initial?: readonly string[]): UseExpandableListReturn {
  const [expandedIds, setExpandedIds] = useState<Set<string>>(() => new Set(initial ?? []))

  const isExpanded = useCallback((id: string): boolean => expandedIds.has(id), [expandedIds])

  const toggle = useCallback((id: string): void => {
    setExpandedIds((prev) => {
      const next = new Set(prev)
      if (next.has(id)) {
        next.delete(id)
      } else {
        next.add(id)
      }
      return next
    })
  }, [])

  const expand = useCallback((id: string): void => {
    setExpandedIds((prev) => {
      if (prev.has(id)) return prev
      const next = new Set(prev)
      next.add(id)
      return next
    })
  }, [])

  const collapse = useCallback((id: string): void => {
    setExpandedIds((prev) => {
      if (!prev.has(id)) return prev
      const next = new Set(prev)
      next.delete(id)
      return next
    })
  }, [])

  const collapseAll = useCallback((): void => {
    setExpandedIds(new Set())
  }, [])

  return {
    expandedIds,
    isExpanded,
    toggle,
    expand,
    collapse,
    collapseAll,
    count: expandedIds.size,
  }
}
