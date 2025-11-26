import { Download, FileText, Image, FileCode, FileArchive } from 'lucide-react'
import { formatFileSize } from '@/lib/utils/format'
import type { FilePart } from '@a2a-js/sdk'

interface FileArtifactProps {
  part: FilePart
}

export function FileArtifact({ part }: FileArtifactProps) {
  const file = part.file
  const isImage = file.mimeType?.startsWith('image/')

  const handleDownload = () => {
    if ('bytes' in file && file.bytes) {
      // Create blob from base64
      const byteCharacters = atob(file.bytes)
      const byteNumbers = new Array(byteCharacters.length)
      for (let i = 0; i < byteCharacters.length; i++) {
        byteNumbers[i] = byteCharacters.charCodeAt(i)
      }
      const byteArray = new Uint8Array(byteNumbers)
      const blob = new Blob([byteArray], { type: file.mimeType || 'application/octet-stream' })

      // Create download link
      const url = URL.createObjectURL(blob)
      const a = document.createElement('a')
      a.href = url
      a.download = file.name || 'download'
      document.body.appendChild(a)
      a.click()
      document.body.removeChild(a)
      URL.revokeObjectURL(url)
    } else if ('uri' in file && file.uri) {
      // Open URI in new tab
      window.open(file.uri, '_blank')
    }
  }

  return (
    <div className="border border-primary-10 rounded-lg p-4 bg-surface-variant">
      <div className="flex items-start gap-3">
        <FileIcon mimeType={file.mimeType} />
        <div className="flex-1">
          <div className="font-medium text-primary">{file.name || 'Unnamed file'}</div>
          {file.mimeType && (
            <div className="text-sm text-secondary">{file.mimeType}</div>
          )}
          {'bytes' in file && file.bytes && (
            <div className="text-sm text-secondary">
              {formatFileSize(Math.floor(file.bytes.length * 0.75))}
            </div>
          )}
        </div>
        <button
          onClick={handleDownload}
          className="p-2 bg-surface border border-primary-10 rounded hover:bg-surface-dark hover:border-primary"
          title="Download file"
        >
          <Download className="w-4 h-4 text-primary" />
        </button>
      </div>

      {/* Preview for images */}
      {isImage && 'bytes' in file && file.bytes && (
        <div className="mt-4">
          <img
            src={`data:${file.mimeType};base64,${file.bytes}`}
            alt={file.name || 'Image'}
            className="max-w-full rounded"
          />
        </div>
      )}

      {/* Preview for images from URI */}
      {isImage && 'uri' in file && file.uri && (
        <div className="mt-4">
          <img
            src={file.uri}
            alt={file.name || 'Image'}
            className="max-w-full rounded"
          />
        </div>
      )}
    </div>
  )
}

function FileIcon({ mimeType }: { mimeType?: string }) {
  if (!mimeType) {
    return <FileText className="w-8 h-8 text-disabled" />
  }

  if (mimeType.startsWith('image/')) {
    return <Image className="w-8 h-8 text-primary" />
  }

  if (mimeType.includes('javascript') || mimeType.includes('json') || mimeType.includes('xml')) {
    return <FileCode className="w-8 h-8 text-success" />
  }

  if (mimeType.includes('zip') || mimeType.includes('tar') || mimeType.includes('gz')) {
    return <FileArchive className="w-8 h-8 text-secondary" />
  }

  return <FileText className="w-8 h-8 text-disabled" />
}