import { useEffect, useState, useRef } from 'react'
import { useArtifactStore } from '@/stores/artifact.store'
import type { Artifact } from '@/types/artifact'
import { useShallow } from 'zustand/react/shallow'

interface ArtifactSubscriptionOptions {
  /**
   * Callback when artifact received
   */
  onArtifact: (artifact: Artifact) => void

  /**
   * Callback on timeout
   */
  onTimeout?: () => void

  /**
   * Timeout in ms (default: 30000)
   */
  timeout?: number
}

/**
 * Hook for subscribing to artifact arrivals via SSE.
 *
 * Use this pattern when triggering MCP tool execution and waiting
 * for the artifact to arrive via SSE broadcast.
 *
 * @example
 * const { subscribe, unsubscribe, isWaiting } = useArtifactSubscription({
 *   onArtifact: (artifact) => setMyData(extractData(artifact)),
 *   onTimeout: () => setError('Tool execution timed out')
 * })
 *
 * const executeTool = async () => {
 *   const execId = crypto.randomUUID()
 *   subscribe(execId)
 *   await callTool(endpoint, toolName, args)
 * }
 */
export function useArtifactSubscription(
  options: ArtifactSubscriptionOptions
) {
  const { onArtifact, onTimeout, timeout = 30000 } = options
  const [activeSubscription, setActiveSubscription] = useState<string | null>(null)
  const timeoutRef = useRef<NodeJS.Timeout | undefined | null>(null)

  const artifacts = useArtifactStore(
    useShallow((state) => state.allIds.map(id => state.byId[id]).filter(Boolean))
  )

  useEffect(() => {
    if (!activeSubscription) return

    const matchingArtifact = artifacts.find((a) => {
      const execId = a.metadata?.tool_execution_id
      return execId === activeSubscription
    })

    if (matchingArtifact) {
      if (timeoutRef.current !== null && timeoutRef.current !== undefined) {
        clearTimeout(timeoutRef.current)
      }
      onArtifact(matchingArtifact)
      setActiveSubscription(null)
    }
  }, [artifacts, activeSubscription, onArtifact])

  const subscribe = (executionId: string) => {
    setActiveSubscription(executionId)

    timeoutRef.current = setTimeout(() => {
      onTimeout?.()
      setActiveSubscription(null)
    }, timeout) as NodeJS.Timeout
  }

  const unsubscribe = () => {
    if (timeoutRef.current !== null && timeoutRef.current !== undefined) {
      clearTimeout(timeoutRef.current)
    }
    setActiveSubscription(null)
  }

  useEffect(() => unsubscribe, [])

  return { subscribe, unsubscribe, isWaiting: !!activeSubscription }
}
