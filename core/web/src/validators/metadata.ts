/**
 * Metadata Validators
 *
 * Validates metadata structures for tasks and artifacts.
 * Provides detailed error messages for debugging malformed data.
 */

import { isPlainObject } from '@/utils/type-guards'
import type { TaskMetadata } from '@/types/task'
import type { ArtifactMetadata, ArtifactType } from '@/types/artifact'

/**
 * Custom error for metadata validation failures.
 * Captures the entity ID and specific issue for debugging.
 */
export class MetadataError extends Error {
  entityId: string
  issue: string

  constructor(entityId: string, issue: string) {
    super(`Invalid metadata for ${entityId}: ${issue}`)
    this.name = 'MetadataError'
    this.entityId = entityId
    this.issue = issue
  }
}

const VALID_TASK_TYPES = ['mcp_execution', 'agent_message'] as const
const VALID_ARTIFACT_TYPES = [
  'text',
  'table',
  'chart',
  'form',
  'code',
  'tree',
  'json',
  'markdown',
  'dashboard',
  'presentation_card',
  'list',
  'copy_paste_text',
  'blog',
] as const

/**
 * Validate task metadata structure.
 * Throws MetadataError if validation fails.
 *
 * @param value - The raw metadata value
 * @param taskId - Task ID for error messages
 * @returns Validated TaskMetadata
 */
export function validateTaskMetadata(value: unknown, taskId: string): TaskMetadata {
  if (!isPlainObject(value)) {
    throw new MetadataError(taskId, 'metadata is not an object')
  }

  if (typeof value.task_type !== 'string') {
    throw new MetadataError(taskId, `missing task_type, got: ${JSON.stringify(value.task_type)}`)
  }

  if (!VALID_TASK_TYPES.includes(value.task_type as (typeof VALID_TASK_TYPES)[number])) {
    throw new MetadataError(
      taskId,
      `invalid task_type: "${value.task_type}", expected one of: ${VALID_TASK_TYPES.join(', ')}`
    )
  }

  if (typeof value.agent_name !== 'string') {
    throw new MetadataError(taskId, `missing agent_name, got: ${JSON.stringify(value.agent_name)}`)
  }

  if (typeof value.created_at !== 'string') {
    throw new MetadataError(taskId, `missing created_at, got: ${JSON.stringify(value.created_at)}`)
  }

  if ('execution_time_ms' in value && value.execution_time_ms !== undefined) {
    if (typeof value.execution_time_ms !== 'number') {
      throw new MetadataError(
        taskId,
        `execution_time_ms must be number, got: ${typeof value.execution_time_ms}`
      )
    }
  }

  if ('input_tokens' in value && value.input_tokens !== undefined) {
    if (typeof value.input_tokens !== 'number') {
      throw new MetadataError(taskId, `input_tokens must be number, got: ${typeof value.input_tokens}`)
    }
  }

  if ('output_tokens' in value && value.output_tokens !== undefined) {
    if (typeof value.output_tokens !== 'number') {
      throw new MetadataError(
        taskId,
        `output_tokens must be number, got: ${typeof value.output_tokens}`
      )
    }
  }

  if ('started_at' in value && value.started_at !== undefined) {
    if (typeof value.started_at !== 'string') {
      throw new MetadataError(taskId, `started_at must be string, got: ${typeof value.started_at}`)
    }
  }

  if ('completed_at' in value && value.completed_at !== undefined) {
    if (typeof value.completed_at !== 'string') {
      throw new MetadataError(
        taskId,
        `completed_at must be string, got: ${typeof value.completed_at}`
      )
    }
  }

  if ('executionSteps' in value && value.executionSteps !== undefined) {
    if (!Array.isArray(value.executionSteps)) {
      throw new MetadataError(
        taskId,
        `executionSteps must be array, got: ${typeof value.executionSteps}`
      )
    }
  }

  return value as TaskMetadata
}

/**
 * Validate artifact metadata structure.
 * Throws MetadataError if validation fails.
 *
 * @param value - The raw metadata value
 * @param artifactId - Artifact ID for error messages
 * @returns Validated ArtifactMetadata
 */
export function validateArtifactMetadata(value: unknown, artifactId: string): ArtifactMetadata {
  if (!isPlainObject(value)) {
    throw new MetadataError(artifactId, 'metadata is not an object')
  }

  if (typeof value.artifact_type !== 'string') {
    throw new MetadataError(
      artifactId,
      `missing artifact_type, got: ${JSON.stringify(value.artifact_type)}`
    )
  }

  const isKnownType = VALID_ARTIFACT_TYPES.includes(value.artifact_type as ArtifactType)
  if (!isKnownType) {
    console.warn(
      `Unknown artifact_type "${value.artifact_type}" for ${artifactId}. Known types: ${VALID_ARTIFACT_TYPES.join(', ')}`
    )
  }

  if (typeof value.created_at !== 'string') {
    throw new MetadataError(
      artifactId,
      `missing created_at, got: ${JSON.stringify(value.created_at)}`
    )
  }

  const isEphemeral = value.ephemeral === true

  if (isEphemeral) {
    if (value.source !== 'mcp_tool') {
      throw new MetadataError(
        artifactId,
        `ephemeral artifact must have source='mcp_tool', got: ${JSON.stringify(value.source)}`
      )
    }
    if (typeof value.tool_name !== 'string') {
      throw new MetadataError(
        artifactId,
        `ephemeral artifact missing tool_name, got: ${JSON.stringify(value.tool_name)}`
      )
    }
    if (typeof value.mcp_execution_id !== 'string') {
      throw new MetadataError(
        artifactId,
        `ephemeral artifact missing mcp_execution_id, got: ${JSON.stringify(value.mcp_execution_id)}`
      )
    }
  } else {
    if (typeof value.context_id !== 'string') {
      throw new MetadataError(
        artifactId,
        `persisted artifact missing context_id, got: ${JSON.stringify(value.context_id)}`
      )
    }
  }

  if ('rendering_hints' in value && value.rendering_hints !== undefined) {
    if (!isPlainObject(value.rendering_hints) && value.rendering_hints !== null) {
      throw new MetadataError(
        artifactId,
        `rendering_hints must be object or null, got: ${typeof value.rendering_hints}`
      )
    }
  }

  return value as ArtifactMetadata
}

/**
 * Type guard to check if task metadata is valid without throwing.
 * Useful for filtering operations.
 *
 * @param value - The raw metadata value
 * @returns True if metadata is valid
 */
export function isValidTaskMetadata(value: unknown): value is TaskMetadata {
  if (!isPlainObject(value)) return false

  return (
    typeof value.task_type === 'string' &&
    VALID_TASK_TYPES.includes(value.task_type as (typeof VALID_TASK_TYPES)[number]) &&
    typeof value.agent_name === 'string' &&
    typeof value.created_at === 'string'
  )
}

/**
 * Type guard to check if artifact metadata is valid without throwing.
 * Useful for filtering operations.
 *
 * @param value - The raw metadata value
 * @returns True if metadata is valid
 */
export function isValidArtifactMetadata(value: unknown): value is ArtifactMetadata {
  if (!isPlainObject(value)) return false

  if (typeof value.artifact_type !== 'string') return false
  if (typeof value.created_at !== 'string') return false

  const isEphemeral = value.ephemeral === true
  if (isEphemeral) {
    return (
      value.source === 'mcp_tool' &&
      typeof value.tool_name === 'string' &&
      typeof value.mcp_execution_id === 'string'
    )
  } else {
    return typeof value.context_id === 'string'
  }
}

/**
 * Safe metadata extraction with default values.
 * Returns a minimal valid metadata object if validation fails.
 *
 * @param value - The raw metadata value
 * @param entityId - Entity ID for logging
 * @param defaults - Default values to use
 * @returns TaskMetadata (validated or defaults)
 */
export function extractTaskMetadataSafe(
  value: unknown,
  entityId: string,
  defaults: { task_type: 'mcp_execution' | 'agent_message'; agent_name: string }
): TaskMetadata {
  try {
    return validateTaskMetadata(value, entityId)
  } catch (e) {
    console.warn(`Using default metadata for ${entityId}:`, e)
    return {
      task_type: defaults.task_type,
      agent_name: defaults.agent_name,
      created_at: new Date().toISOString(),
    }
  }
}
