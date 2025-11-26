import type { Artifact as A2AArtifact, Part } from '@a2a-js/sdk'

export type RenderBehavior = 'modal' | 'inline' | 'silent' | 'both'

interface BaseArtifactMetadata {
  artifact_type: string
  created_at: string
  source?: string
  tool_name?: string
  mcp_execution_id?: string
  [k: string]: unknown
}

export interface EphemeralArtifactMetadata extends BaseArtifactMetadata {
  ephemeral: true
  source: 'mcp_tool'
  tool_name: string
  mcp_execution_id: string
}

export interface PersistedArtifactMetadata extends BaseArtifactMetadata {
  ephemeral?: false
  context_id: string
  task_id?: string
  render_behavior?: RenderBehavior
}

export type ArtifactMetadata = EphemeralArtifactMetadata | PersistedArtifactMetadata

export type Artifact = Omit<A2AArtifact, 'metadata'> & {
  metadata: ArtifactMetadata
}

export type EphemeralArtifact = Omit<A2AArtifact, 'metadata'> & {
  metadata: EphemeralArtifactMetadata
}

export type PersistedArtifact = Omit<A2AArtifact, 'metadata'> & {
  metadata: PersistedArtifactMetadata
}

export function isEphemeralArtifact(artifact: Artifact): artifact is EphemeralArtifact {
  return artifact.metadata.ephemeral === true
}

export function isPersistedArtifact(artifact: Artifact): artifact is PersistedArtifact {
  return !artifact.metadata.ephemeral
}

export interface StreamingArtifactState {
  isAppending: boolean
  isComplete: boolean
  previousParts: Part[]
}

export function validateArtifact(artifact: A2AArtifact): artifact is Artifact {
  if (!artifact.metadata) {
    return false
  }

  const metadata = artifact.metadata as Record<string, unknown>
  return (
    typeof metadata.artifact_type === 'string' &&
    typeof metadata.context_id === 'string' &&
    typeof metadata.created_at === 'string'
  )
}

export function toArtifact(artifact: A2AArtifact): Artifact {
  if (!validateArtifact(artifact)) {
    throw new Error(
      `Invalid artifact: missing required metadata fields. ` +
      `Expected: artifact_type, context_id, created_at. ` +
      `Received: ${JSON.stringify(artifact.metadata)}`
    )
  }
  return artifact
}

/**
 * Type guard to check if an artifact has a valid artifactId
 */
export function hasArtifactId(artifact: unknown): artifact is { artifactId: string } & Record<string, unknown> {
  return (
    typeof artifact === 'object' &&
    artifact !== null &&
    'artifactId' in artifact &&
    typeof (artifact as { artifactId: unknown }).artifactId === 'string'
  )
}

/**
 * Type guard to check if artifact metadata has a fingerprint
 */
export function hasFingerprint(artifact: A2AArtifact): boolean {
  return (
    typeof artifact.metadata === 'object' &&
    artifact.metadata !== null &&
    'fingerprint' in artifact.metadata &&
    typeof (artifact.metadata as { fingerprint: unknown }).fingerprint === 'string'
  )
}

/**
 * Safely extract artifactId from an A2A artifact
 */
export function getArtifactId(artifact: A2AArtifact): string | undefined {
  if (hasArtifactId(artifact)) {
    return artifact.artifactId
  }
  return undefined
}

/**
 * Safely extract fingerprint from artifact metadata
 */
export function getFingerprint(artifact: A2AArtifact): string | undefined {
  if (hasFingerprint(artifact)) {
    return (artifact.metadata as { fingerprint: string }).fingerprint
  }
  return undefined
}
