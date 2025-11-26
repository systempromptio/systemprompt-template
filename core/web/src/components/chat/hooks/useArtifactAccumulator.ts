/**
 * Hook for accumulating and managing artifacts during streaming.
 *
 * Handles the complex logic of artifact deduplication, persistence
 * to the store, and final artifact creation after streaming completes.
 *
 * @module chat/hooks/useArtifactAccumulator
 */

import { useCallback } from 'react'
import { useArtifactStore } from '@/stores/artifact.store'
import type { Artifact } from '@/types/artifact'

/**
 * Artifact accumulator hook return value.
 */
interface UseArtifactAccumulatorReturn {
  /**
   * Finalizes accumulated artifacts and persists them to the store
   */
  finalizeArtifacts: (
    artifacts: Map<string, Artifact>,
    contextId: string,
    messageId: string
  ) => Promise<void>
}

/**
 * Accumulates artifacts during streaming and persists them to the store.
 *
 * Handles:
 * - Deduplication based on artifact ID
 * - Merging partial artifact updates
 * - Final persistence after stream completes
 *
 * @returns Artifact accumulation functions
 *
 * @example
 * ```typescript
 * function ChatInterface() {
 *   const { finalizeArtifacts } = useArtifactAccumulator()
 *
 *   const handleStreamComplete = async (artifacts: Map<string, Artifact>) => {
 *     await finalizeArtifacts(artifacts, contextId, messageId)
 *   }
 * }
 * ```
 */
export function useArtifactAccumulator(): UseArtifactAccumulatorReturn {
  const addArtifact = useArtifactStore((state) => state.addArtifact)

  /**
   * Finalizes accumulated artifacts and persists them to the store.
   *
   * @param artifacts - Map of accumulated artifacts from streaming
   * @param contextId - The conversation context ID
   * @param messageId - The message these artifacts belong to
   */
  const finalizeArtifacts = useCallback(
    async (
      artifacts: Map<string, Artifact>,
      contextId: string,
      messageId: string
    ) => {
      for (const [_artifactId, artifact] of artifacts.entries()) {
        addArtifact(artifact, messageId, contextId)
      }
    },
    [addArtifact]
  )

  return { finalizeArtifacts }
}
