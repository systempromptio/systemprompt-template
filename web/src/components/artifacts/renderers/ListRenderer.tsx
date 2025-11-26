import { ExternalLink, AlertTriangle } from 'lucide-react'
import type { Artifact } from '@/types/artifact'
import { extractListData, type ListItem } from '@/lib/artifacts'

interface ListRendererProps {
  artifact: Artifact
}

function ListItemCard({ item }: { item: ListItem }) {
  return (
    <div className="border border-primary-10 rounded-lg p-4 bg-surface hover:bg-surface-variant transition-fast group">
      <div className="flex items-start justify-between gap-3">
        <div className="flex-1 min-w-0">
          <h3 className="font-medium text-primary mb-1 group-hover:text-accent transition-fast">
            {item.title}
          </h3>
          {item.summary && (
            <p className="text-sm text-secondary line-clamp-2">
              {item.summary}
            </p>
          )}
        </div>
        {item.link && (
          <a
            href={item.link}
            target="_blank"
            rel="noopener noreferrer"
            className="flex-shrink-0 p-2 hover:bg-surface-dark rounded transition-fast"
            title="Open link"
          >
            <ExternalLink className="w-4 h-4 text-primary group-hover:text-accent transition-fast" />
          </a>
        )}
      </div>
    </div>
  )
}

export function ListRenderer({ artifact }: ListRendererProps) {
  const result = extractListData(artifact)
  const { items, count } = result.data
  const errors = result.errors

  if (errors && errors.length > 0) {
    return (
      <div className="space-y-3">
        <div className="flex items-center gap-2 text-error text-sm p-3 bg-error/10 border border-error/20 rounded">
          <AlertTriangle className="w-4 h-4" />
          <div>
            {errors.map((error, idx) => (
              <div key={idx}>{error}</div>
            ))}
          </div>
        </div>
      </div>
    )
  }

  if (items.length === 0) {
    return (
      <div className="text-secondary text-sm italic py-4 text-center">
        No items found
      </div>
    )
  }

  return (
    <div className="space-y-3">
      {items.map((item, idx) => (
        <ListItemCard key={idx} item={item} />
      ))}

      {count !== undefined && (
        <div className="text-xs text-secondary text-center pt-2">
          Showing {items.length} of {count} items
        </div>
      )}
    </div>
  )
}
