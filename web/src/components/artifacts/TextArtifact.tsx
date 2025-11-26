import { useState, lazy, Suspense } from 'react'
import { Copy, Check } from 'lucide-react'
import type { TextPart } from '@a2a-js/sdk'

const MarkdownContent = lazy(() => import('@/components/markdown/MarkdownContent').then(m => ({ default: m.MarkdownContent })))

interface TextArtifactProps {
  part: TextPart
}

export function TextArtifact({ part }: TextArtifactProps) {
  const [copied, setCopied] = useState(false)

  const handleCopy = () => {
    navigator.clipboard.writeText(part.text)
    setCopied(true)
    setTimeout(() => setCopied(false), 2000)
  }

  // Check if content looks like code
  const isCode = part.text.includes('```') ||
                 part.text.includes('function') ||
                 part.text.includes('const ') ||
                 part.text.includes('import ')

  return (
    <div className="relative">
      <button
        onClick={handleCopy}
        className="absolute top-2 right-2 p-1.5 bg-surface border border-primary-10 rounded hover:bg-surface-variant"
        title="Copy to clipboard"
      >
        {copied ? (
          <Check className="w-4 h-4 text-success" />
        ) : (
          <Copy className="w-4 h-4 text-secondary" />
        )}
      </button>

      {isCode ? (
        <pre className="bg-surface-variant p-4 rounded overflow-x-auto text-sm text-primary">
          <code>{part.text}</code>
        </pre>
      ) : (
        <Suspense fallback={<div className="text-text-secondary">{part.text}</div>}>
          <MarkdownContent content={part.text} />
        </Suspense>
      )}
    </div>
  )
}