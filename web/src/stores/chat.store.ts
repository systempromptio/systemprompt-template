import { create } from 'zustand'
import type { Part } from '@a2a-js/sdk'
import type { Task } from '@/types/task'
import type { Artifact, StreamingArtifactState } from '@/types/artifact'

/**
 * Valid task state values for validation
 */
const VALID_TASK_STATES = [
  'submitted',
  'working',
  'input-required',
  'completed',
  'canceled',
  'failed',
  'rejected',
  'auth-required',
  'unknown'
] as const

/**
 * Represents a single message in the chat interface
 */
export interface ChatMessage {
  id: string
  timestamp: Date
  role: 'user' | 'assistant'
  content: string
  agentId?: string
  parts?: Part[]
  artifacts?: Artifact[]
  streamingArtifacts?: Map<string, StreamingArtifactState>
  contextId?: string
  isStreaming?: boolean
  task?: Task
  toolCalls?: ToolCallExecution[]
  metadata?: Record<string, any>
  agenticExecution?: {
    currentIteration?: number
  }
}

/**
 * Represents the execution details of a tool call
 */
export interface ToolCallExecution {
  id: string
  name: string
  arguments: any
  result?: {
    is_error: boolean
    content: any
  }
  timestamp: Date
}

/**
 * Represents a pending request for user input
 */
export interface InputRequest {
  taskId: string
  messageId: string
  message?: any
  timestamp: Date
}

/**
 * Represents a pending authentication request
 */
export interface AuthRequest {
  taskId: string
  messageId: string
  message?: any
  timestamp: Date
}

/**
 * Helper function to ensure task ID exists in array
 *
 * @param taskId - The task ID to check
 * @param taskIds - Current array of task IDs
 * @returns Updated array with task ID included if it wasn't already present
 */
const ensureTaskIdInArray = (taskId: string, taskIds: readonly string[]): readonly string[] => {
  return taskIds.includes(taskId) ? taskIds : [...taskIds, taskId]
}

/**
 * Helper function to remove an entry from a record and array
 *
 * @param byId - Record mapping IDs to items
 * @param ids - Array of IDs
 * @param idToRemove - ID to remove
 * @returns Updated record and array without the specified ID
 */
const removeRequestById = <T>(
  byId: Record<string, T>,
  ids: readonly string[],
  idToRemove: string
): { byId: Record<string, T>; ids: readonly string[] } => {
  const { [idToRemove]: _, ...remainingRequests } = byId
  const remainingIds = ids.filter(id => id !== idToRemove)
  return { byId: remainingRequests, ids: remainingIds }
}

/**
 * Zustand store interface for managing chat state
 */
export interface ChatStore {
  tasksById: Record<string, Task>
  taskIds: readonly string[]
  currentStreamingMessageId: string | null
  pendingInputRequestsById: Record<string, InputRequest>
  pendingInputRequestIds: readonly string[]
  pendingAuthRequestsById: Record<string, AuthRequest>
  pendingAuthRequestIds: readonly string[]

  updateTask: (taskId: string, task: Task) => void
  registerInputRequest: (taskId: string, messageId: string, message?: any) => void
  registerAuthRequest: (taskId: string, messageId: string, message?: any) => void
  resolveInputRequest: (taskId: string) => void
  resolveAuthRequest: (taskId: string) => void
  setStreamingMessage: (id: string | null) => void
  handleTaskStatusChanged?: (event: { context_id: string; task_id: string; status: string; timestamp: string }) => void
  reset: () => void
}

/**
 * Zustand store for managing chat state including tasks and pending requests
 *
 * State is organized as:
 * - tasksById: Record mapping task IDs to task objects
 * - taskIds: Array of all task IDs
 * - currentStreamingMessageId: ID of currently streaming message
 * - pendingInputRequestsById: Record of pending user input requests by task ID
 * - pendingInputRequestIds: Array of task IDs with pending input requests
 * - pendingAuthRequestsById: Record of pending auth requests by task ID
 * - pendingAuthRequestIds: Array of task IDs with pending auth requests
 *
 * This store manages the runtime state of chat interactions, tracking
 * active tasks and pending user interactions
 */
export const useChatStore = create<ChatStore>()((set, get) => ({
  tasksById: {},
  taskIds: [],
  currentStreamingMessageId: null,
  pendingInputRequestsById: {},
  pendingInputRequestIds: [],
  pendingAuthRequestsById: {},
  pendingAuthRequestIds: [],

  /**
   * Updates or adds a task to the store
   *
   * @param taskId - The task ID to update
   * @param task - The complete task object
   */
  updateTask: (taskId, task) => {
    set((state) => ({
      tasksById: { ...state.tasksById, [taskId]: task },
      taskIds: ensureTaskIdInArray(taskId, state.taskIds)
    }))
  },

  /**
   * Registers a new pending input request from a task
   *
   * @param taskId - The task requesting input
   * @param messageId - The message ID associated with the request
   * @param message - Optional message content
   */
  registerInputRequest: (taskId, messageId, message) => {
    set((state) => ({
      pendingInputRequestsById: {
        ...state.pendingInputRequestsById,
        [taskId]: { taskId, messageId, message, timestamp: new Date() }
      },
      pendingInputRequestIds: ensureTaskIdInArray(taskId, state.pendingInputRequestIds)
    }))
  },

  /**
   * Registers a new pending authentication request from a task
   *
   * @param taskId - The task requesting authentication
   * @param messageId - The message ID associated with the request
   * @param message - Optional message content
   */
  registerAuthRequest: (taskId, messageId, message) => {
    set((state) => ({
      pendingAuthRequestsById: {
        ...state.pendingAuthRequestsById,
        [taskId]: { taskId, messageId, message, timestamp: new Date() }
      },
      pendingAuthRequestIds: ensureTaskIdInArray(taskId, state.pendingAuthRequestIds)
    }))
  },

  /**
   * Resolves and removes a pending input request
   *
   * @param taskId - The task ID whose input request should be resolved
   */
  resolveInputRequest: (taskId) => {
    set((state) => {
      const { byId, ids } = removeRequestById(
        state.pendingInputRequestsById,
        state.pendingInputRequestIds,
        taskId
      )
      return {
        pendingInputRequestsById: byId,
        pendingInputRequestIds: ids
      }
    })
  },

  /**
   * Resolves and removes a pending authentication request
   *
   * @param taskId - The task ID whose auth request should be resolved
   */
  resolveAuthRequest: (taskId) => {
    set((state) => {
      const { byId, ids } = removeRequestById(
        state.pendingAuthRequestsById,
        state.pendingAuthRequestIds,
        taskId
      )
      return {
        pendingAuthRequestsById: byId,
        pendingAuthRequestIds: ids
      }
    })
  },

  /**
   * Sets the currently streaming message ID
   *
   * @param id - The message ID that is currently streaming, or null if no message is streaming
   */
  setStreamingMessage: (id: string | null) => {
    set({ currentStreamingMessageId: id })
  },

  /**
   * Handles a task status changed event from the server
   * Updates the task status if the task exists and the new status is valid
   *
   * @param event - The status change event containing context_id, task_id, status, and timestamp
   */
  handleTaskStatusChanged: (event) => {
    const task = get().tasksById[event.task_id]
    if (!task) {
      return
    }

    if (!VALID_TASK_STATES.includes(event.status as any)) {
      return
    }

    set((state) => ({
      tasksById: {
        ...state.tasksById,
        [event.task_id]: {
          ...task,
          status: {
            ...task.status,
            state: event.status as typeof task.status.state,
            timestamp: event.timestamp
          }
        }
      }
    }))
  },

  /**
   * Resets the entire store to initial state
   */
  reset: () => {
    set({
      tasksById: {},
      taskIds: [],
      currentStreamingMessageId: null,
      pendingInputRequestsById: {},
      pendingInputRequestIds: [],
      pendingAuthRequestsById: {},
      pendingAuthRequestIds: [],
    })
  },
}))

/**
 * Selector functions for accessing chat store state
 */
export const chatSelectors = {
  /**
   * Gets a task by its ID
   *
   * @param state - Chat store state
   * @param taskId - Task ID to look up
   * @returns Task object if found, undefined otherwise
   */
  getTaskById: (state: ChatStore, taskId: string): Task | undefined =>
    state.tasksById[taskId],

  /**
   * Gets the ID of the currently streaming message
   *
   * @param state - Chat store state
   * @returns Message ID if a message is streaming, null otherwise
   */
  getCurrentStreamingMessageId: (state: ChatStore): string | null =>
    state.currentStreamingMessageId ?? null,

  /**
   * Gets a pending input request for a specific task
   *
   * @param state - Chat store state
   * @param taskId - Task ID to get input request for
   * @returns InputRequest if found, undefined otherwise
   */
  getInputRequest: (state: ChatStore, taskId: string): InputRequest | undefined =>
    state.pendingInputRequestsById[taskId],

  /**
   * Gets a pending authentication request for a specific task
   *
   * @param state - Chat store state
   * @param taskId - Task ID to get auth request for
   * @returns AuthRequest if found, undefined otherwise
   */
  getAuthRequest: (state: ChatStore, taskId: string): AuthRequest | undefined =>
    state.pendingAuthRequestsById[taskId],

  /**
   * Checks if there are any pending input requests
   *
   * @param state - Chat store state
   * @returns True if at least one input request is pending, false otherwise
   */
  hasPendingInputRequests: (state: ChatStore): boolean =>
    state.pendingInputRequestIds.length > 0,

  /**
   * Checks if there are any pending authentication requests
   *
   * @param state - Chat store state
   * @returns True if at least one auth request is pending, false otherwise
   */
  hasPendingAuthRequests: (state: ChatStore): boolean =>
    state.pendingAuthRequestIds.length > 0,

  /**
   * Gets all pending input request IDs
   *
   * @param state - Chat store state
   * @returns Array of task IDs with pending input requests
   */
  getPendingInputRequestIds: (state: ChatStore): readonly string[] =>
    state.pendingInputRequestIds,

  /**
   * Gets all pending authentication request IDs
   *
   * @param state - Chat store state
   * @returns Array of task IDs with pending auth requests
   */
  getPendingAuthRequestIds: (state: ChatStore): readonly string[] =>
    state.pendingAuthRequestIds,

  /**
   * Gets the total count of tasks in the store
   *
   * @param state - Chat store state
   * @returns Number of tasks
   */
  getTaskCount: (state: ChatStore): number => state.taskIds.length,

  /**
   * Checks if a message is currently streaming
   *
   * @param state - Chat store state
   * @returns True if a message is streaming, false otherwise
   */
  hasStreamingMessage: (state: ChatStore): boolean =>
    state.currentStreamingMessageId !== null,
}
