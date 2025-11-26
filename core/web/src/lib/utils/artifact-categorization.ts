/**
 * Artifact categorization utilities.
 *
 * Separates artifacts into categories for display purposes.
 *
 * @module lib/utils/artifact-categorization
 */

import type { Artifact } from '@/types/artifact'
import type { Part } from '@a2a-js/sdk'

/**
 * Categorized artifacts grouped by display type.
 */
export interface CategorizedArtifacts {
  /**
   * Internal system artifacts (hidden by default)
   */
  internal: Artifact[]

  /**
   * Tool execution result artifacts
   */
  toolExecution: Artifact[]

  /**
   * Prominent user-facing artifacts
   */
  prominent: Artifact[]
}

/**
 * Categorizes artifacts for display in message bubbles.
 *
 * Separates artifacts into:
 * - Internal: marked with is_internal=true
 * - ToolExecution: artifact_type === 'tool_execution'
 * - Prominent: all others (main content artifacts)
 *
 * @param artifacts - Array of artifacts to categorize
 * @returns Categorized artifacts object
 *
 * @example
 * ```typescript
 * const { prominent, toolExecution, internal } = categorizeArtifacts(
 *   message.artifacts || []
 * )
 * ```
 */
export function categorizeArtifacts(artifacts: Artifact[]): CategorizedArtifacts {
  const internal: Artifact[] = []
  const toolExecution: Artifact[] = []
  const prominent: Artifact[] = []

  for (const artifact of artifacts) {
    if (artifact.metadata?.is_internal === true) {
      internal.push(artifact)
    } else if (artifact.metadata?.artifact_type === 'tool_execution') {
      toolExecution.push(artifact)
    } else {
      prominent.push(artifact)
    }
  }

  return { internal, toolExecution, prominent }
}

/**
 * Extracts artifact ID from message part data.
 *
 * Looks for artifactId in:
 * 1. part.data.artifactId
 * 2. part.data.metadata.artifactId
 *
 * @param data - Part data object
 * @returns Artifact ID or undefined
 */
export function extractArtifactId(data: unknown): string | undefined {
  if (typeof data === 'object' && data !== null) {
    const obj = data as Record<string, unknown>
    if (typeof obj.artifactId === 'string') {
      return obj.artifactId
    }
    if (typeof obj.metadata === 'object' && obj.metadata !== null) {
      const metadata = obj.metadata as Record<string, unknown>
      if (typeof metadata.artifactId === 'string') {
        return metadata.artifactId
      }
    }
  }
  return undefined
}

/**
 * Gets artifact IDs from message parts to prevent duplicate rendering.
 *
 * Scans all data parts and collects artifact IDs to avoid showing
 * the same artifact twice (once in parts, once in artifacts array).
 *
 * @param parts - Message parts array
 * @returns Set of artifact IDs found in data parts
 */
export function getArtifactIdsFromParts(parts: Part[] | undefined): Set<string> {
  const ids = new Set<string>()

  if (!parts) return ids

  for (const part of parts) {
    if (part.kind === 'data' && part.data) {
      const id = extractArtifactId(part.data)
      if (id) {
        ids.add(id)
      }
    }
  }

  return ids
}
