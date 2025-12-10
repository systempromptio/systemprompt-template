import { useState } from 'react'
import type { Artifact, PresentationHints } from '@/types/artifact'
import { useA2AClient } from '@/hooks/useA2AClient'
import { extractPresentationCardData } from '@/lib/artifacts/extractors'
import { cn } from '@/lib/utils/cn'
import { CardHeader } from './presentation/CardHeader'
import { CardSections } from './presentation/CardSections'
import { CardCTAs } from './presentation/CardCTAs'

interface PresentationCardRendererProps {
  artifact: Artifact
  hints: PresentationHints
}

function getThemeClasses(theme?: string) {
  switch (theme) {
    case 'gradient':
    case 'accent':
      return cn('bg-gradient-to-br', 'from-[var(--card-gradient-start)] via-[var(--card-gradient-mid)] to-[var(--card-gradient-end)]')
    case 'glass':
      return cn(
        'bg-gradient-to-br',
        'from-[rgba(var(--color-primary-rgb),0.08)] via-[rgba(var(--color-primary-rgb),0.05)] to-[rgba(var(--color-primary-rgb),0.03)]',
        'backdrop-blur-md'
      )
    case 'minimal':
    case 'default':
    default:
      return 'bg-surface'
  }
}

export function PresentationCardRenderer({ artifact, hints }: PresentationCardRendererProps) {
  const { sendMessage } = useA2AClient()
  const [clickingId, setClickingId] = useState<string | null>(null)
  const [successId, setSuccessId] = useState<string | null>(null)

  const result = extractPresentationCardData(artifact)
  const cardData = result.data
  const theme = cardData.theme || hints.theme || 'gradient'

  // Extract metadata from artifact
  const metadata = artifact.metadata as Record<string, unknown>
  const skillName = metadata?.skill_name as string | undefined
  const artifactId = artifact.artifactId
  const taskId = metadata?.task_id as string | undefined
  const contextId = metadata?.context_id as string | undefined
  const createdAt = metadata?.created_at as string | undefined

  const handleCTAClick = async (ctaId: string, message: string) => {
    if (!sendMessage) {
      alert('Unable to send message: Chat client not initialized')
      return
    }

    setClickingId(ctaId)
    try {
      await sendMessage(message)
      setClickingId(null)
      setSuccessId(ctaId)
      setTimeout(() => setSuccessId(null), 1500)
    } catch (error) {
      setClickingId(null)
      const errorMsg = error instanceof Error ? error.message : 'Failed to send message'
      alert(`Error: ${errorMsg}`)
    }
  }

  return (
    <div
      className={cn(
        'rounded-[var(--card-radius-default)] rounded-tr-[var(--card-radius-cut)]',
        'border border-primary-10',
        'shadow-[var(--card-shadow-md)]',
        'transition-all duration-[var(--animation-normal)]',
        'hover:shadow-[var(--card-shadow-lg)] hover:scale-[1.01]',
        'overflow-hidden',
        getThemeClasses(theme)
      )}
    >
      {/* Skill and Metadata Header */}
      <div className="px-lg py-md border-b border-primary-10 bg-surface/30">
        {/* Skill Badge Row */}
        <div className="flex items-center gap-sm mb-sm">
          <span className="text-sm text-text-tertiary font-medium">Skill used:</span>
          {skillName ? (
            <span className="inline-flex items-center gap-1.5 px-3 py-1 rounded-full bg-primary/10 text-primary font-semibold text-sm border border-primary/20">
              <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9.663 17h4.673M12 3v1m6.364 1.636l-.707.707M21 12h-1M4 12H3m3.343-5.657l-.707-.707m2.828 9.9a5 5 0 117.072 0l-.548.547A3.374 3.374 0 0014 18.469V19a2 2 0 11-4 0v-.531c0-.895-.356-1.754-.988-2.386l-.548-.547z" />
              </svg>
              {skillName}
            </span>
          ) : (
            <span className="text-text-tertiary italic text-sm">none</span>
          )}
        </div>

        {/* IDs Grid */}
        <div className="grid grid-cols-1 gap-y-1 text-xs">
          <div className="flex items-baseline gap-2">
            <span className="text-text-tertiary font-medium min-w-[60px]">artifact:</span>
            <span className="font-mono text-text-secondary break-all">{artifactId}</span>
          </div>
          {taskId && (
            <div className="flex items-baseline gap-2">
              <span className="text-text-tertiary font-medium min-w-[60px]">task:</span>
              <span className="font-mono text-text-secondary break-all">{taskId}</span>
            </div>
          )}
          {contextId && (
            <div className="flex items-baseline gap-2">
              <span className="text-text-tertiary font-medium min-w-[60px]">context:</span>
              <span className="font-mono text-text-secondary break-all">{contextId}</span>
            </div>
          )}
          {createdAt && (
            <div className="flex items-baseline gap-2">
              <span className="text-text-tertiary font-medium min-w-[60px]">created:</span>
              <span className="text-text-secondary">{new Date(createdAt).toLocaleString()}</span>
            </div>
          )}
        </div>
      </div>

      <CardHeader cardData={cardData} />
      <CardSections sections={cardData.sections ?? []} />
      <CardCTAs ctas={cardData.ctas ?? []} clickingId={clickingId} successId={successId} onCTAClick={handleCTAClick} />
    </div>
  )
}
