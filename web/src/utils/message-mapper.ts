/**
 * Message Mapper Utilities
 *
 * Transforms task data from A2A protocol into chat messages with deduplication.
 * Handles artifact association, message grouping, and filtering of system messages.
 *
 * @example
 * ```typescript
 * const messages = mapTasksToChatMessages(tasks, contextId)
 * console.log(messages.length) // Unique deduplicated messages
 * ```
 */

import type { Task as A2ATask } from '@a2a-js/sdk'
import type { Task } from '@/types/task'
import type { ChatMessage } from '@/stores/chat.store'
import { toTask } from '@/types/task'
import { toArtifact } from '@/types/artifact'
import { logger } from '@/lib/logger'

export type { ChatMessage }

/**
 * Identify tool execution system messages that should be filtered out
 * @internal
 */
const isToolExecutionInfoMessage = (content: string): boolean => {
  const patterns = [
    /^Tool execution completed/i,
    /^Executed MCP tool:/i,
    /^Error executing/i,
    /^Created artifact:/i,
    /^Execution ID:/i,
  ]
  return patterns.some(pattern => pattern.test(content.trim()))
}

/**
 * Generate stable hash for message content deduplication
 * @param message - Chat message to hash
 * @returns Stable hash string based on role, content, and context
 * @throws Error if message is missing required fields
 */
export const getMessageContentHash = (message: ChatMessage): string => {
  if (!message) throw new Error('getMessageContentHash: message is null/undefined')
  if (message.role === null || message.role === undefined) throw new Error(`getMessageContentHash: message missing role: ${JSON.stringify(message)}`)
  if (message.content === null || message.content === undefined) throw new Error(`getMessageContentHash: message missing content: ${JSON.stringify(message)}`)

  const normalizedContent = message.content.trim()
  const key = `${message.role}:${normalizedContent}:${message.contextId || 'nocontext'}`
  let hash = 0
  for (let i = 0; i < key.length; i++) {
    const char = key.charCodeAt(i)
    hash = ((hash << 5) - hash) + char
    hash = hash & hash
  }
  return `hash-${Math.abs(hash)}`
}

/**
 * Convert task history to chat messages
 *
 * Extracts message history from task, associates artifacts with messages,
 * and filters out tool execution system messages.
 *
 * @param task - Task containing message history
 * @param contextId - Context UUID for message grouping
 * @returns Array of chat messages derived from task history
 *
 * @example
 * ```typescript
 * const messages = mapTaskHistoryToChatMessages(task, contextId)
 * ```
 */
export const mapTaskHistoryToChatMessages = (task: Task, contextId: string): ChatMessage[] => {
  if (!task.history || task.history.length === 0) {
    return []
  }

  const history = task.history

  const validatedArtifacts = task.artifacts
    ?.map(a => {
      try {
        return toArtifact(a)
      } catch (e) {
        logger.warn('Skipping invalid artifact', e, 'message-mapper')
        return null
      }
    })
    .filter((a): a is NonNullable<typeof a> => a !== null)

  return history
    .map((msg, index) => {
      const textPart = msg.parts?.find((p) => 'text' in p && p.kind === 'text') as { text?: string } | undefined
      const content = textPart?.text || ''

      const createdAt = new Date(task.metadata.created_at)
      const createdAtMs = createdAt.getTime()

      const isLastMessage = index === history.length - 1
      const isAssistantMessage = msg.role === 'agent'
      const role: 'user' | 'assistant' = msg.role === 'agent' ? 'assistant' : 'user'

      return {
        id: msg.messageId,
        timestamp: new Date(createdAtMs + index),
        role,
        content,
        parts: msg.parts || [],
        artifacts: (isLastMessage && isAssistantMessage) ? validatedArtifacts : undefined,
        contextId,
        task: (isLastMessage && isAssistantMessage) ? task : undefined,
        metadata: msg.metadata,
      }
    })
    .filter(message => {
      if (message.role === 'assistant' && isToolExecutionInfoMessage(message.content)) {
        logger.debug('Filtering out tool execution info message', message.content.substring(0, 100), 'message-mapper')
        return false
      }
      // Filter out empty messages with completed tasks (placeholder messages)
      if (message.role === 'assistant' && !message.content.trim() && message.task?.status?.state === 'completed') {
        logger.debug('Filtering out empty completed task message', { taskId: message.task?.id }, 'message-mapper')
        return false
      }
      return true
    })
}

/**
 * Convert array of tasks to deduplicated chat messages
 *
 * Transforms multiple tasks into a unified message stream with deduplication
 * by content hash. Messages are sorted chronologically and filtered to remove
 * system messages.
 *
 * @param tasks - Array of A2A task objects
 * @param contextId - Context UUID for message grouping
 * @returns Deduplicated messages sorted by timestamp
 * @throws Error if task conversion fails
 *
 * @example
 * ```typescript
 * const messages = mapTasksToChatMessages(tasks, contextId)
 * ```
 */
export const mapTasksToChatMessages = (tasks: A2ATask[], contextId: string): ChatMessage[] => {
  const validatedTasks = tasks
    .map((t, idx) => {
      try {
        return toTask(t)
      } catch (e) {
        throw new Error(`mapTasksToChatMessages: Failed to convert task at index ${idx}: ${e instanceof Error ? e.message : String(e)}`)
      }
    })

  const allMessages = validatedTasks.flatMap((task, idx) => {
    try {
      return mapTaskHistoryToChatMessages(task, contextId)
    } catch (e) {
      throw new Error(`mapTasksToChatMessages: Failed to map history for task ${task.id} (index ${idx}): ${e instanceof Error ? e.message : String(e)}`)
    }
  })

  const seenIds = new Set<string>()
  const dedupedMessages = allMessages.filter((msg, idx) => {
    if (seenIds.has(msg.id)) {
      logger.debug('Removing duplicate message', { index: idx, id: msg.id }, 'message-mapper')
      return false
    }
    seenIds.add(msg.id)
    return true
  })

  // Final filter to remove any empty completed task messages that slipped through
  const finalMessages = dedupedMessages.filter(msg => {
    if (msg.role === 'assistant' && !msg.content.trim() && msg.task?.status?.state === 'completed') {
      logger.debug('Filtering out empty completed task message in final pass', { taskId: msg.task?.id }, 'message-mapper')
      return false
    }
    return true
  })

  return finalMessages.sort((a, b) => {
    const aTime = a.timestamp instanceof Date ? a.timestamp.getTime() : new Date(a.timestamp).getTime()
    const bTime = b.timestamp instanceof Date ? b.timestamp.getTime() : new Date(b.timestamp).getTime()
    return aTime - bTime
  })
}
