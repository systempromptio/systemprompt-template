import { Copy, Check, AlertTriangle } from 'lucide-react'
import { useState } from 'react'
import type { Artifact } from '@/types/artifact'

interface CopyPasteTextRendererProps {
  artifact: Artifact
}

export function CopyPasteTextRenderer({ artifact }: CopyPasteTextRendererProps) {
  const [copied, setCopied] = useState(false)

  const dataPart = artifact.parts.find(p => p.kind === 'data')
  if (!dataPart || dataPart.kind !== 'data') {
    return (
      <div className="flex items-center gap-3 p-4 bg-error/10 border border-error/20 rounded-lg">
        <AlertTriangle className="w-5 h-5 text-error flex-shrink-0" />
        <span className="text-sm text-error">No content found</span>
      </div>
    )
  }

  const data = dataPart.data as Record<string, unknown>
  const content = data.content as string
  const title = data.title as string | undefined
  const language = data.language as string | undefined

  if (!content || typeof content !== 'string') {
    return (
      <div className="flex items-center gap-3 p-4 bg-error/10 border border-error/20 rounded-lg">
        <AlertTriangle className="w-5 h-5 text-error flex-shrink-0" />
        <span className="text-sm text-error">Invalid content format</span>
      </div>
    )
  }

  const handleCopy = () => {
    navigator.clipboard.writeText(content)
    setCopied(true)
    setTimeout(() => setCopied(false), 2000)
  }

  return (
    <div className="border border-primary-10 rounded-lg overflow-hidden bg-surface">
      {/* Header with title, language, and copy button */}
      <div className="flex items-center justify-between px-4 py-3 bg-surface-variant border-b border-primary-10 gap-3">
        <div className="flex items-center gap-3 min-w-0">
          {title && (
            <span className="text-sm font-medium text-primary truncate">{title}</span>
          )}
          {language && (
            <span className="text-xs text-secondary px-2 py-1 bg-surface rounded-md flex-shrink-0">
              {language}
            </span>
          )}
        </div>
        <button
          onClick={handleCopy}
          className="flex items-center gap-2 px-3 py-1.5 bg-primary text-white rounded-md hover:bg-primary-dark transition-colors flex-shrink-0 text-sm font-medium"
          title="Copy to clipboard"
        >
          {copied ? (
            <>
              <Check className="w-4 h-4" />
              <span>Copied!</span>
            </>
          ) : (
            <>
              <Copy className="w-4 h-4" />
              <span>Copy</span>
            </>
          )}
        </button>
      </div>

      {/* Content */}
      <div className="p-4 bg-surface overflow-x-auto">
        <pre className="text-sm font-mono text-primary whitespace-pre-wrap break-words leading-relaxed">
          <code>{content}</code>
        </pre>
      </div>
    </div>
  )
}
