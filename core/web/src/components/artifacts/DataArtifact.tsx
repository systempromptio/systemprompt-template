import { useState } from 'react'
import { Copy, Check, ChevronDown, ChevronRight } from 'lucide-react'
import type { DataPart } from '@a2a-js/sdk'

interface DataArtifactProps {
  part: DataPart
}

export function DataArtifact({ part }: DataArtifactProps) {
  const [copied, setCopied] = useState(false)
  const [expanded, setExpanded] = useState(true)

  const handleCopy = () => {
    navigator.clipboard.writeText(JSON.stringify(part.data, null, 2))
    setCopied(true)
    setTimeout(() => setCopied(false), 2000)
  }

  return (
    <div className="border border-primary-10 rounded-lg bg-surface-variant">
      <div className="flex items-center justify-between px-4 py-2 border-b border-primary-10 bg-surface">
        <button
          onClick={() => setExpanded(!expanded)}
          className="flex items-center gap-1 text-sm font-medium text-primary"
        >
          {expanded ? (
            <ChevronDown className="w-4 h-4" />
          ) : (
            <ChevronRight className="w-4 h-4" />
          )}
          Data Object
        </button>
        <button
          onClick={handleCopy}
          className="p-1.5 bg-surface-variant border border-primary-10 rounded hover:bg-surface-dark"
          title="Copy JSON"
        >
          {copied ? (
            <Check className="w-3 h-3 text-success" />
          ) : (
            <Copy className="w-3 h-3 text-secondary" />
          )}
        </button>
      </div>

      {expanded && (
        <div className="p-4">
          {isSimpleObject(part.data) ? (
            <SimpleDataView data={part.data as Record<string, string | number | boolean | null>} />
          ) : (
            <pre className="text-sm overflow-x-auto">
              <code>{JSON.stringify(part.data, null, 2)}</code>
            </pre>
          )}
        </div>
      )}
    </div>
  )
}

function isSimpleObject(data: unknown): boolean {
  if (typeof data !== 'object' || data === null) return false
  if (Array.isArray(data)) return false

  // Check if all values are primitives
  return Object.values(data).every(
    (value) =>
      typeof value === 'string' ||
      typeof value === 'number' ||
      typeof value === 'boolean' ||
      value === null
  )
}

function SimpleDataView({ data }: { data: Record<string, string | number | boolean | null> }) {
  return (
    <table className="w-full text-sm">
      <tbody>
        {Object.entries(data).map(([key, value], idx) => (
          <tr
            key={key}
            className={idx % 2 === 0 ? 'bg-surface' : 'bg-surface-variant'}
          >
            <td className="px-3 py-1.5 font-medium text-secondary w-1/3">
              {key}
            </td>
            <td className="px-3 py-1.5 text-primary">
              {value === null ? (
                <span className="text-disabled italic">null</span>
              ) : typeof value === 'boolean' ? (
                <span className={value ? 'text-success' : 'text-error'}>
                  {String(value)}
                </span>
              ) : (
                String(value)
              )}
            </td>
          </tr>
        ))}
      </tbody>
    </table>
  )
}