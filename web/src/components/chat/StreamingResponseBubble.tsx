/**
 * Streaming response bubble for showing execution progress.
 *
 * Displays as an AI message bubble while the agent is processing
 * but before any text response is available.
 */

import { Card } from '@/components/ui'
import { Avatar } from '@/components/ui/Avatar'
import { ExecutionTimeline } from './ExecutionTimeline'
import type { ExecutionStep } from '@/types/execution'

interface StreamingResponseBubbleProps {
  executionSteps: ExecutionStep[]
  agentName?: string
  agentId?: string
}

export function StreamingResponseBubble({
  executionSteps,
  agentName,
  agentId,
}: StreamingResponseBubbleProps) {
  return (
    <div className="flex gap-3 animate-slideInUp max-w-full flex-row">
      <Avatar
        variant="agent"
        agentName={agentName}
        agentId={agentId}
        size="sm"
      />

      <div className="flex-1 min-w-0">
        <Card
          variant="accent"
          padding="md"
          elevation="sm"
          cutCorner="top-left"
          className="font-body max-w-full overflow-hidden"
        >
          <ExecutionTimeline mode="streaming" steps={executionSteps} variant="bubble" />
        </Card>
      </div>
    </div>
  )
}
