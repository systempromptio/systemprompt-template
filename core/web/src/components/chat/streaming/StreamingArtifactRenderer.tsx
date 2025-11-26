import { useEffect, useState, useRef } from 'react'
import { cn } from '@/lib/utils/cn'
import type { Part } from '@a2a-js/sdk'
import type { Artifact } from '@/types/artifact'
import { ArtifactViewer } from '@/components/artifacts/ArtifactViewer'

interface StreamingArtifactRendererProps {
  artifact: Artifact
  isAppending?: boolean
  isComplete?: boolean
  previousParts?: Part[]
}

export function StreamingArtifactRenderer({
  artifact,
  isAppending = false,
  isComplete = true,
  previousParts = []
}: StreamingArtifactRendererProps) {
  const [displayedParts, setDisplayedParts] = useState<Part[]>(artifact.parts)
  const [newPartIndices, setNewPartIndices] = useState<Set<number>>(new Set())
  const previousPartsLengthRef = useRef(artifact.parts.length)

  useEffect(() => {
    const currentLength = artifact.parts.length
    const previousLength = previousParts.length

    if (isAppending && previousLength > 0 && currentLength > previousPartsLengthRef.current) {
      const newPartsStartIndex = previousLength
      const newIndices = new Set<number>()

      for (let i = newPartsStartIndex; i < currentLength; i++) {
        newIndices.add(i)
      }

      setNewPartIndices(newIndices)
      setDisplayedParts(artifact.parts)
      previousPartsLengthRef.current = currentLength

      const timer = setTimeout(() => {
        setNewPartIndices(new Set())
      }, 2000)

      return () => clearTimeout(timer)
    } else if (currentLength !== previousPartsLengthRef.current) {
      setDisplayedParts(artifact.parts)
      previousPartsLengthRef.current = currentLength
    }
  }, [artifact.parts.length, isAppending, previousParts.length])

  const displayArtifact = {
    ...artifact,
    parts: displayedParts
  }

  return (
    <div className="relative">
      <div
        className={cn(
          'transition-all duration-300',
          !isComplete && 'opacity-90'
        )}
      >
        <ArtifactViewer artifact={displayArtifact} />
      </div>

      {!isComplete && (
        <div className="absolute top-3 right-3">
          <div className="px-2 py-1 text-xs bg-blue-500 text-white rounded-full shadow-sm flex items-center gap-1.5">
            <div className="w-1.5 h-1.5 bg-white rounded-full animate-pulse" />
            Updating
          </div>
        </div>
      )}

      {isAppending && newPartIndices.size > 0 && (
        <div className="absolute -right-2 top-1/2 -translate-y-1/2">
          <div className="px-2 py-1 text-xs bg-green-500 text-white rounded-l-full shadow-md animate-slideInRight">
            +{newPartIndices.size} new
          </div>
        </div>
      )}
    </div>
  )
}

export function ArtifactStreamingWrapper({ children, isNew = false }: { children: React.ReactNode; isNew?: boolean }) {
  const [shouldAnimate, setShouldAnimate] = useState(isNew)

  useEffect(() => {
    if (isNew) {
      setShouldAnimate(true)
      const timer = setTimeout(() => setShouldAnimate(false), 3000)
      return () => clearTimeout(timer)
    }
  }, [isNew])

  return (
    <div className={cn(
      'transition-all duration-500',
      shouldAnimate && 'bg-green-50 shadow-lg'
    )}>
      {children}
    </div>
  )
}
