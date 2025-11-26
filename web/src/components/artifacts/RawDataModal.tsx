import { X, Copy, Check } from 'lucide-react'
import { useState } from 'react'
import { StreamingData } from '../chat/streaming/StreamingData'
import type { Artifact } from '@a2a-js/sdk'

interface RawDataModalProps {
  artifact: Artifact
  isOpen: boolean
  onClose: () => void
}

export function RawDataModal({ artifact, isOpen, onClose }: RawDataModalProps) {
  const [copied, setCopied] = useState(false)

  if (!isOpen) return null

  // Extract structured data from artifact parts
  const dataPart = artifact.parts.find(part => part.kind === 'data')
  const structuredData = dataPart?.kind === 'data' ? dataPart.data : {}

  const handleCopy = () => {
    const jsonString = JSON.stringify(structuredData, null, 2)
    navigator.clipboard.writeText(jsonString)
    setCopied(true)
    setTimeout(() => setCopied(false), 2000)
  }

  return (
    <div className="fixed inset-0 z-modal flex items-center justify-center bg-black/50 backdrop-blur-sm">
      <div className="bg-surface border border-primary-10 rounded-lg shadow-lg w-[90vw] h-[80vh] max-w-4xl flex flex-col">
        {/* Header */}
        <div className="flex items-center justify-between px-4 py-3 border-b border-primary-10">
          <h3 className="text-lg font-medium text-primary">Raw Structured Data</h3>
          <div className="flex items-center gap-2">
            <button
              onClick={handleCopy}
              className="p-2 hover:bg-surface-variant rounded transition-fast"
              title="Copy JSON"
            >
              {copied ? (
                <Check className="w-4 h-4 text-success" />
              ) : (
                <Copy className="w-4 h-4 text-secondary" />
              )}
            </button>
            <button
              onClick={onClose}
              className="p-2 hover:bg-surface-variant rounded transition-fast"
              title="Close"
            >
              <X className="w-4 h-4 text-secondary" />
            </button>
          </div>
        </div>

        {/* Content */}
        <div className="flex-1 overflow-auto p-4">
          <StreamingData data={structuredData} isComplete={true} />
        </div>
      </div>
    </div>
  )
}
