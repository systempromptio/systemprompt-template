/**
 * Stream event processing utilities for chat messages.
 *
 * Handles different types of streaming events (messages, artifacts, tasks)
 * and applies the appropriate state updates.
 *
 * @module chat/helpers/streamEventHandlers
 */

import { isMessageEvent, extractTextContent } from './typeGuards'
import type { Artifact as A2AArtifact } from '@a2a-js/sdk'
import type { Artifact, StreamingArtifactState } from '@/types/artifact'
import { toArtifact, getArtifactId, getFingerprint } from '@/types/artifact'

/**
 * Processes a message event from the stream.
 *
 * Accumulates text content from agent message parts into a single
 * streaming message buffer.
 *
 * @param event - The message event to process
 * @param currentText - Current accumulated text
 * @returns Updated text content
 */
export function processMessageEvent(
  event: Record<string, unknown>,
  currentText: string
): string {
  if (!isMessageEvent(event) || event.role !== 'agent' || !event.parts) {
    return currentText
  }

  const textContent = extractTextContent(event.parts as Array<{ kind: string; text?: string }>)
  return currentText + textContent
}

/**
 * Processes an artifact update event from the stream.
 *
 * Accumulates artifact data in a Map structure, merging updates
 * for existing artifacts or creating new entries.
 *
 * @param event - The artifact update event
 * @param artifacts - Current artifacts Map
 * @param artifactRegistry - Registry of fingerprints to artifact IDs
 * @param streamingArtifactsState - Streaming state for artifacts
 * @returns Object with updated artifacts map and state
 */
export function processArtifactEvent(
  event: Record<string, unknown>,
  artifacts: Map<string, Artifact>,
  artifactRegistry: Map<string, string>,
  streamingArtifactsState: Map<string, StreamingArtifactState>
): { artifacts: Map<string, Artifact>; registry: Map<string, string>; state: Map<string, StreamingArtifactState> } {
  const rawArtifact = event.artifact as A2AArtifact | undefined

  if (!rawArtifact) {
    return { artifacts, registry: artifactRegistry, state: streamingArtifactsState }
  }

  const artifactId = getArtifactId(rawArtifact)
  const fingerprint = getFingerprint(rawArtifact)

  if (!artifactId) {
    return { artifacts, registry: artifactRegistry, state: streamingArtifactsState }
  }

  const updatedArtifacts = new Map(artifacts)
  const updatedRegistry = new Map(artifactRegistry)
  const updatedState = new Map(streamingArtifactsState)

  // Handle deduplication if fingerprint exists
  if (fingerprint && updatedRegistry.has(fingerprint)) {
    const oldArtifactId = updatedRegistry.get(fingerprint)!
    updatedArtifacts.delete(oldArtifactId)
    updatedState.delete(oldArtifactId)
  }

  if (fingerprint) {
    updatedRegistry.set(fingerprint, artifactId)
  }

  // Validate and merge artifact
  let validatedArtifact: Artifact
  try {
    validatedArtifact = toArtifact(rawArtifact)
  } catch (e) {
    return { artifacts, registry: artifactRegistry, state: streamingArtifactsState }
  }

  const existing = updatedArtifacts.get(artifactId)
  if (existing) {
    updatedArtifacts.set(artifactId, {
      ...existing,
      parts: [...(existing.parts || []), ...(validatedArtifact.parts || [])],
    })
  } else {
    updatedArtifacts.set(artifactId, validatedArtifact)
  }

  updatedState.set(artifactId, {
    isAppending: true,
    isComplete: false,
    previousParts: updatedArtifacts.get(artifactId)?.parts || [],
  })

  return { artifacts: updatedArtifacts, registry: updatedRegistry, state: updatedState }
}

/**
 * Finalizes streaming artifact states.
 *
 * Marks all streaming artifacts as complete.
 *
 * @param streamingArtifactsState - Map of artifact streaming states
 * @returns Updated streaming state with isComplete set to true
 */
export function finalizeStreamingArtifacts(
  streamingArtifactsState: Map<string, StreamingArtifactState>
): Map<string, StreamingArtifactState> {
  const updated = new Map(streamingArtifactsState)
  updated.forEach((state, artifactId) => {
    updated.set(artifactId, { ...state, isComplete: true })
  })
  return updated
}
