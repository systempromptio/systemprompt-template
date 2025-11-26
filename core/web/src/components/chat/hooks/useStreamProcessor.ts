/**
 * Hook for processing streaming events from the chat API.
 *
 * Handles the real-time stream of events (messages, artifacts, tasks)
 * and maintains the accumulated state during streaming.
 *
 * @module chat/hooks/useStreamProcessor
 */

import { useState, useCallback, useRef } from 'react'
import { isMessageEvent, isArtifactUpdateEvent } from '../helpers/typeGuards'
import { processMessageEvent, processArtifactEvent, finalizeStreamingArtifacts } from '../helpers/streamEventHandlers'
import type { Artifact, StreamingArtifactState } from '@/types/artifact'

/**
 * Stream processing state.
 */
interface StreamState {
  /**
   * Accumulated text content from the stream
   */
  text: string

  /**
   * Accumulated artifacts from the stream (keyed by artifact ID)
   */
  artifacts: Map<string, Artifact>

  /**
   * Artifact fingerprint registry for deduplication
   */
  artifactRegistry: Map<string, string>

  /**
   * Streaming state for each artifact
   */
  streamingArtifactsState: Map<string, StreamingArtifactState>

  /**
   * Whether the stream is currently active
   */
  isStreaming: boolean
}

/**
 * Stream processor hook return value.
 */
interface UseStreamProcessorReturn {
  /**
   * Current stream state
   */
  state: StreamState

  /**
   * Processes a single stream event and updates state
   */
  processEvent: (event: unknown) => void

  /**
   * Resets the stream state to initial values
   */
  reset: () => void

  /**
   * Sets streaming status
   */
  setStreaming: (isStreaming: boolean) => void

  /**
   * Finalizes streaming artifacts
   */
  finalizeArtifacts: () => void

  /**
   * Gets current stream state snapshot
   */
  getStreamState: () => {
    text: string
    artifacts: Map<string, Artifact>
    streamingArtifactsState: Map<string, StreamingArtifactState>
  }
}

/**
 * Processes streaming events from the chat API.
 *
 * Maintains accumulated state for messages, artifacts, and tasks
 * during a streaming response.
 *
 * @returns Stream processor state and controls
 *
 * @example
 * ```typescript
 * function ChatInterface() {
 *   const { state, processEvent, reset, finalizeArtifacts } = useStreamProcessor()
 *
 *   const handleStream = async (text: string) => {
 *     reset()
 *     for await (const event of streamMessage(text)) {
 *       processEvent(event)
 *     }
 *     finalizeArtifacts()
 *   }
 *
 *   return <div>{state.text}</div>
 * }
 * ```
 */
export function useStreamProcessor(): UseStreamProcessorReturn {
  const [state, setState] = useState<StreamState>({
    text: '',
    artifacts: new Map(),
    artifactRegistry: new Map(),
    streamingArtifactsState: new Map(),
    isStreaming: false,
  })

  // Use ref to track current state for synchronous reads (avoids stale closure)
  const stateRef = useRef<StreamState>(state)
  stateRef.current = state

  const processEvent = useCallback((event: unknown) => {
    setState((prev) => {
      if (isMessageEvent(event) && (event as Record<string, unknown>).role === 'agent') {
        const newText = processMessageEvent(event as Record<string, unknown>, prev.text)
        const newState = { ...prev, text: newText }
        stateRef.current = newState
        return newState
      } else if (isArtifactUpdateEvent(event)) {
        const { artifacts, registry, state: newState } = processArtifactEvent(
          event as Record<string, unknown>,
          prev.artifacts,
          prev.artifactRegistry,
          prev.streamingArtifactsState
        )
        const updatedState = {
          ...prev,
          artifacts,
          artifactRegistry: registry,
          streamingArtifactsState: newState,
        }
        stateRef.current = updatedState
        return updatedState
      }
      return prev
    })
  }, [])

  const reset = useCallback(() => {
    const resetState = {
      text: '',
      artifacts: new Map(),
      artifactRegistry: new Map(),
      streamingArtifactsState: new Map(),
      isStreaming: false,
    }
    stateRef.current = resetState
    setState(resetState)
  }, [])

  const setStreaming = useCallback((isStreaming: boolean) => {
    setState((prev) => {
      const newState = { ...prev, isStreaming }
      stateRef.current = newState
      return newState
    })
  }, [])

  const finalizeArtifacts = useCallback(() => {
    setState((prev) => {
      const newState = {
        ...prev,
        streamingArtifactsState: finalizeStreamingArtifacts(prev.streamingArtifactsState),
      }
      stateRef.current = newState
      return newState
    })
  }, [])

  const getStreamState = useCallback(() => {
    // Read from ref for immediate, synchronous access to current state
    return {
      text: stateRef.current.text,
      artifacts: stateRef.current.artifacts,
      streamingArtifactsState: stateRef.current.streamingArtifactsState,
    }
  }, [])

  return { state, processEvent, reset, setStreaming, finalizeArtifacts, getStreamState }
}
