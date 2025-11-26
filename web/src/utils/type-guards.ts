/**
 * Centralized type guard utilities for discriminated unions.
 *
 * Provides reusable type predicates for chat events, artifacts, tasks,
 * and other discriminated union types used throughout the application.
 *
 * @module utils/type-guards
 */

import type { Task } from '@/types/task'
import type { TaskStatus } from '@a2a-js/sdk'
import type { Artifact, EphemeralArtifact } from '@/types/artifact'

/**
 * Check if an artifact is persisted (stored in database).
 *
 * Distinguishes between persisted artifacts and ephemeral runtime artifacts.
 *
 * @param artifact - The artifact to check
 * @returns True if artifact has an ID and is persisted
 *
 * @example
 * ```typescript
 * if (isPersistedArtifact(artifact)) {
 *   // Safe to delete or update in database
 * }
 * ```
 */
export function isPersistedArtifact(artifact: unknown): artifact is Artifact {
  if (!artifact || typeof artifact !== 'object') return false

  const obj = artifact as Record<string, unknown>
  return (
    typeof obj.id === 'string' &&
    typeof obj.name === 'string' &&
    typeof obj.description === 'string'
  )
}

/**
 * Check if an artifact is ephemeral (runtime-only).
 *
 * Ephemeral artifacts exist only in memory and are not persisted.
 *
 * @param artifact - The artifact to check
 * @returns True if artifact is ephemeral
 *
 * @example
 * ```typescript
 * if (isEphemeralArtifact(artifact)) {
 *   // Will not be saved to database
 * }
 * ```
 */
export function isEphemeralArtifact(artifact: unknown): artifact is EphemeralArtifact {
  if (!artifact || typeof artifact !== 'object') return false

  const obj = artifact as Record<string, unknown>
  const metadata = obj.metadata as Record<string, unknown> | undefined
  return metadata?.ephemeral === true
}

/**
 * Check if a value is a valid task status state.
 *
 * Validates against known task state values.
 *
 * @param value - The value to check
 * @returns True if value is a valid task state
 *
 * @example
 * ```typescript
 * if (isValidTaskState(userInput)) {
 *   // Safe to use as task state
 * }
 * ```
 */
export function isValidTaskState(value: unknown): value is string {
  const validStates = [
    'submitted',
    'working',
    'input-required',
    'completed',
    'failed',
    'rejected',
    'canceled',
    'auth-required',
  ]

  return typeof value === 'string' && validStates.includes(value)
}

/**
 * Check if a task is in a terminal state.
 *
 * Terminal states are states where the task will not progress further.
 *
 * @param task - The task to check
 * @returns True if task is in a terminal state
 *
 * @example
 * ```typescript
 * if (isTerminalTask(task)) {
 *   // Task execution is complete
 * }
 * ```
 */
export function isTerminalTask(task: Task | null | undefined): task is Task {
  if (!task?.status) return false

  const terminalStates = ['completed', 'failed', 'rejected', 'canceled']
  return terminalStates.includes(task.status.state)
}

/**
 * Check if a task is currently active/running.
 *
 * @param task - The task to check
 * @returns True if task is actively running
 *
 * @example
 * ```typescript
 * if (isRunningTask(task)) {
 *   // Show loading indicator
 * }
 * ```
 */
export function isRunningTask(task: Task | null | undefined): task is Task {
  if (!task?.status) return false

  const runningStates = ['submitted', 'working', 'input-required']
  return runningStates.includes(task.status.state)
}

/**
 * Check if a task has failed.
 *
 * Covers both explicit failures and rejections.
 *
 * @param task - The task to check
 * @returns True if task failed
 *
 * @example
 * ```typescript
 * if (isFailedTask(task)) {
 *   // Show error message
 * }
 * ```
 */
export function isFailedTask(task: Task | null | undefined): task is Task {
  if (!task?.status) return false

  return ['failed', 'rejected'].includes(task.status.state)
}

/**
 * Check if a task requires user input.
 *
 * @param task - The task to check
 * @returns True if task is waiting for input
 *
 * @example
 * ```typescript
 * if (isInputRequiredTask(task)) {
 *   // Show input dialog
 * }
 * ```
 */
export function isInputRequiredTask(task: Task | null | undefined): task is Task {
  return task?.status?.state === 'input-required'
}

/**
 * Check if a task requires authentication.
 *
 * @param task - The task to check
 * @returns True if task needs auth
 *
 * @example
 * ```typescript
 * if (isAuthRequiredTask(task)) {
 *   // Show login dialog
 * }
 * ```
 */
export function isAuthRequiredTask(task: Task | null | undefined): task is Task {
  return task?.status?.state === 'auth-required'
}

/**
 * Check if a task has metadata attached.
 *
 * @param task - The task to check
 * @returns True if task has metadata
 *
 * @example
 * ```typescript
 * if (hasTaskMetadata(task)) {
 *   const agentName = task.metadata.agent_name
 * }
 * ```
 */
export function hasTaskMetadata(task: Task | null | undefined): boolean {
  return (
    task?.metadata !== null && task?.metadata !== undefined
  )
}

/**
 * Type guard to narrow unknown to Task.
 *
 * @param value - The value to check
 * @returns True if value is a Task object
 *
 * @example
 * ```typescript
 * const items = fetchItems()
 * const tasks = items.filter(isTask)
 * ```
 */
export function isTask(value: unknown): value is Task {
  if (!value || typeof value !== 'object') return false

  const obj = value as Record<string, unknown>
  return (
    typeof obj.id === 'string' &&
    obj.status !== null &&
    typeof obj.status === 'object'
  )
}

/**
 * Type guard to narrow unknown to TaskStatus.
 *
 * @param value - The value to check
 * @returns True if value is a TaskStatus object
 *
 * @example
 * ```typescript
 * if (isTaskStatus(obj)) {
 *   console.log(obj.state)
 * }
 * ```
 */
export function isTaskStatus(value: unknown): value is TaskStatus {
  if (!value || typeof value !== 'object') return false

  const obj = value as Record<string, unknown>
  return (
    typeof obj.state === 'string' &&
    typeof obj.timestamp === 'string'
  )
}

/**
 * Check if a value is a non-empty string.
 *
 * Useful for filtering out empty values.
 *
 * @param value - The value to check
 * @returns True if value is a non-empty string
 *
 * @example
 * ```typescript
 * const filledStrings = values.filter(isNonEmptyString)
 * ```
 */
export function isNonEmptyString(value: unknown): value is string {
  return typeof value === 'string' && value.length > 0
}

/**
 * Check if a value is a valid UUID.
 *
 * Uses basic UUID format validation (v4 style).
 *
 * @param value - The value to check
 * @returns True if value looks like a UUID
 *
 * @example
 * ```typescript
 * if (isUUID(id)) {
 *   // Safe to use as unique identifier
 * }
 * ```
 */
export function isUUID(value: unknown): value is string {
  if (typeof value !== 'string') return false

  const uuidRegex = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i
  return uuidRegex.test(value)
}

/**
 * Check if a value is a valid URL.
 *
 * @param value - The value to check
 * @returns True if value is a valid URL
 *
 * @example
 * ```typescript
 * if (isValidURL(endpoint)) {
 *   // Safe to use for fetch
 * }
 * ```
 */
export function isValidURL(value: unknown): value is string {
  if (typeof value !== 'string') return false

  try {
    new URL(value)
    return true
  } catch {
    return false
  }
}

/**
 * Check if a value is a valid email address.
 *
 * Uses basic email format validation.
 *
 * @param value - The value to check
 * @returns True if value looks like an email
 *
 * @example
 * ```typescript
 * if (isValidEmail(userInput)) {
 *   // Safe to send
 * }
 * ```
 */
export function isValidEmail(value: unknown): value is string {
  if (typeof value !== 'string') return false

  const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/
  return emailRegex.test(value)
}

/**
 * Check if a value is null or undefined.
 *
 * Useful for filtering out nullable values.
 *
 * @param value - The value to check
 * @returns True if value is null or undefined
 *
 * @example
 * ```typescript
 * const defined = values.filter(v => !isNullish(v))
 * ```
 */
export function isNullish(value: unknown): value is null | undefined {
  return value === null || value === undefined
}

/**
 * Check if a value is a plain object (not array, null, etc).
 *
 * @param value - The value to check
 * @returns True if value is a plain object
 *
 * @example
 * ```typescript
 * if (isPlainObject(data)) {
 *   // Safe to access properties
 * }
 * ```
 */
export function isPlainObject(value: unknown): value is Record<string, unknown> {
  return value !== null && typeof value === 'object' && !Array.isArray(value)
}

/**
 * Check if an error is an instance of Error.
 *
 * @param error - The error to check
 * @returns True if error is an Error instance
 *
 * @example
 * ```typescript
 * try {
 *   // Do something
 * } catch (err) {
 *   if (isError(err)) {
 *     console.log(err.message)
 *   }
 * }
 * ```
 */
export function isError(error: unknown): error is Error {
  return error instanceof Error
}
