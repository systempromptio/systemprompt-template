import { useState, useEffect } from 'react'
import { FileText, Download, Image as ImageIcon } from 'lucide-react'
import { cn } from '@/lib/utils/cn'
import type { FileWithBytes, FileWithUri } from '@a2a-js/sdk'

interface StreamingFileProps {
  file: FileWithBytes | FileWithUri
  isComplete?: boolean
}

export function StreamingFile({ file, isComplete = true }: StreamingFileProps) {
  const [loaded, setLoaded] = useState(false)
  const [imageError, setImageError] = useState(false)

  const isImage = file.mimeType?.startsWith('image/')
  const fileUrl = 'uri' in file && file.uri
    ? file.uri
    : 'bytes' in file && file.bytes
    ? `data:${file.mimeType};base64,${file.bytes}`
    : null

  useEffect(() => {
    if (isComplete) {
      setLoaded(true)
    }
  }, [isComplete])

  if (!isComplete) {
    return (
      <div className="p-4 bg-white/10 rounded-lg animate-pulse">
        <div className="flex items-center gap-3">
          <div className="w-10 h-10 bg-gray-300 rounded" />
          <div className="flex-1 space-y-2">
            <div className="h-4 bg-gray-300 rounded w-3/4" />
            <div className="h-3 bg-gray-300 rounded w-1/2" />
          </div>
        </div>
      </div>
    )
  }

  return (
    <div
      className={cn(
        'p-3 bg-white/10 rounded-lg border border-gray-200 transition-all',
        loaded ? 'opacity-100' : 'opacity-0'
      )}
    >
      {isImage && fileUrl && !imageError ? (
        <div className="space-y-2">
          <img
            src={fileUrl}
            alt={file.name || 'Image'}
            className="max-w-full h-auto rounded"
            onLoad={() => setLoaded(true)}
            onError={() => {
              setImageError(true)
              setLoaded(true)
            }}
          />
          <div className="flex items-center justify-between text-xs text-gray-600">
            <span>{file.name || 'image.png'}</span>
            {file.mimeType && <span className="opacity-70">{file.mimeType}</span>}
          </div>
        </div>
      ) : (
        <div className="flex items-center gap-3">
          <div className="p-2 bg-gray-100 rounded">
            {isImage ? (
              <ImageIcon className="w-6 h-6 text-gray-600" />
            ) : (
              <FileText className="w-6 h-6 text-gray-600" />
            )}
          </div>
          <div className="flex-1 min-w-0">
            <div className="font-medium text-sm truncate">
              {file.name || 'File'}
            </div>
            {file.mimeType && (
              <div className="text-xs text-gray-500">{file.mimeType}</div>
            )}
          </div>
          {fileUrl && (
            <a
              href={fileUrl}
              download={file.name}
              className="p-2 hover:bg-gray-100 rounded transition-colors"
            >
              <Download className="w-4 h-4 text-gray-600" />
            </a>
          )}
        </div>
      )}
    </div>
  )
}
