import React from 'react'
import { StreamingText } from './streaming/StreamingText'
import { StreamingFile } from './streaming/StreamingFile'
import { StreamingData } from './streaming/StreamingData'
import type { Part } from '@a2a-js/sdk'

interface PartRendererProps {
  part: Part
  isStreaming?: boolean
  isUser?: boolean
}

export const PartRenderer = React.memo(function PartRenderer({
  part,
  isStreaming = false,
  isUser = false,
}: PartRendererProps) {
  if (part.kind === 'text') {
    return (
      <StreamingText
        text={part.text || ''}
        isUser={isUser}
        isStreaming={isStreaming}
      />
    )
  }

  if (part.kind === 'file') {
    return (
      <div className="mt-md">
        <StreamingFile
          file={part.file}
          isComplete={!isStreaming}
        />
      </div>
    )
  }

  if (part.kind === 'data') {
    return (
      <div className="mt-md">
        <StreamingData
          data={part.data}
          isComplete={!isStreaming}
        />
      </div>
    )
  }

  return null
})
