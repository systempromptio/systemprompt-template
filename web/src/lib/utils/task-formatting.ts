/**
 * Task display formatting utilities.
 *
 * Provides helpers for formatting task data for display.
 *
 * @module lib/utils/task-formatting
 */

import type { Task } from '@/types/task'

/**
 * Gets the agent name from a task.
 *
 * @param task - Task object
 * @returns Agent name or empty string
 */
export function getAgentName(task: Task): string {
  return task.metadata.agent_name || ''
}

/**
 * Gets the MCP server name from a task.
 *
 * @param task - Task object
 * @returns MCP server name or '-'
 */
export function getMcpServerName(task: Task): string {
  return task.metadata.mcp_server_name || '-'
}

/**
 * Gets the conversation name from context ID.
 *
 * @param contextId - Context ID
 * @param conversationMap - Map of context IDs to conversation objects
 * @returns Conversation display name with explicit fallback indicator
 */
export function getConversationName(contextId: string, conversationMap: Map<string, { name: string }>): string {
  const conversation = conversationMap.get(contextId)

  if (!conversation) {
    // New or unloaded conversation - show as new with ID prefix
    return `New (${contextId.substring(0, 8)})`
  }

  if (!conversation.name) {
    // Conversation exists but unnamed
    return `Unnamed (${contextId.substring(0, 8)})`
  }

  return conversation.name
}

/**
 * Formats task status with styling information.
 *
 * @param state - Task status state
 * @returns Object with className and display text
 */
export function formatTaskStatus(state: string): {
  className: string
  text: string
} {
  const baseClasses = 'px-xs py-xs rounded text-xs font-medium whitespace-nowrap border'

  switch (state) {
    case 'completed':
      return {
        className: `${baseClasses} bg-success/20 text-success border-success/30`,
        text: state,
      }
    case 'failed':
    case 'rejected':
      return {
        className: `${baseClasses} bg-error/20 text-error border-error/30`,
        text: state,
      }
    case 'working':
      return {
        className: `${baseClasses} bg-warning/20 text-warning border-warning/30`,
        text: state,
      }
    default:
      return {
        className: `${baseClasses} bg-surface-variant text-text-secondary border-primary/10`,
        text: state,
      }
  }
}
