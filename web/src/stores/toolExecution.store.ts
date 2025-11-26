import { create } from 'zustand'
import type { Artifact } from '@/types/artifact'
import type { RenderBehavior } from '@/types/artifact'

/**
 * Represents the state and details of a tool execution
 */
export interface ToolExecution {
  id: string
  toolName: string
  serverName: string
  status: 'pending' | 'executing' | 'completed' | 'error'
  artifact?: Artifact
  error?: string
  timestamp: number
  renderBehavior: RenderBehavior
  parameters?: Record<string, unknown>
  executionTime?: number
}

/**
 * Helper function to check if execution is in active state
 *
 * @param status - The execution status to check
 * @returns True if status is pending or executing, false otherwise
 */
const isActiveStatus = (status: ToolExecution['status']): boolean => {
  return status === 'pending' || status === 'executing'
}

/**
 * Helper function to update a specific execution by ID
 *
 * @param executions - Array of all executions
 * @param id - ID of execution to update
 * @param updates - Partial updates to apply
 * @returns Updated executions array
 */
const updateExecutionById = (
  executions: readonly ToolExecution[],
  id: string,
  updates: Partial<ToolExecution>
): readonly ToolExecution[] => {
  return executions.map((exec) =>
    exec.id === id ? { ...exec, ...updates } : exec
  )
}

/**
 * Zustand store interface for managing tool execution state
 */
interface ToolExecutionStore {
  executions: readonly ToolExecution[]
  addExecution: (execution: ToolExecution) => void
  updateExecution: (id: string, updates: Partial<ToolExecution>) => void
  completeExecution: (id: string, artifact: Artifact) => void
  failExecution: (id: string, error: string) => void
  getQueue: () => ToolExecution[]
  getActive: () => ToolExecution[]
  getById: (id: string) => ToolExecution | undefined
  findByArtifactId: (artifactId: string) => ToolExecution | undefined
  clearCompleted: () => void
  removeExecution: (id: string) => void
}

/**
 * Zustand store for managing tool execution lifecycle
 *
 * State is organized as:
 * - executions: Array of all tool executions (past and present)
 *
 * This store tracks the complete lifecycle of tool executions from
 * initiation through completion or failure. It maintains execution
 * history and provides methods to query executions by various criteria.
 *
 * Execution statuses:
 * - pending: Execution queued but not started
 * - executing: Currently running
 * - completed: Successfully finished with artifact result
 * - error: Failed with error message
 */
export const useToolExecutionStore = create<ToolExecutionStore>((set, get) => ({
  executions: [],

  /**
   * Adds a new tool execution to the store
   * Should be called when a tool execution is initiated
   *
   * @param execution - The tool execution to add
   */
  addExecution: (execution) => {
    set((state) => ({
      executions: [...state.executions, execution],
    }))
  },

  /**
   * Updates an existing tool execution with partial updates
   * Commonly used to update status or add execution details
   *
   * @param id - The execution ID to update
   * @param updates - Partial execution properties to update
   */
  updateExecution: (id, updates) => {
    set((state) => ({
      executions: updateExecutionById(state.executions, id, updates),
    }))
  },

  /**
   * Marks a tool execution as completed with its result artifact
   * Automatically calculates execution time based on timestamp
   *
   * @param id - The execution ID to complete
   * @param artifact - The resulting artifact from the execution
   */
  completeExecution: (id, artifact) => {
    set((state) => ({
      executions: updateExecutionById(state.executions, id, {
        status: 'completed' as const,
        artifact,
        executionTime: Date.now() - (state.executions.find(e => e.id === id)?.timestamp || Date.now()),
      }),
    }))
  },

  /**
   * Marks a tool execution as failed with an error message
   * Automatically calculates execution time based on timestamp
   *
   * @param id - The execution ID to mark as failed
   * @param error - Error message describing the failure
   */
  failExecution: (id, error) => {
    set((state) => ({
      executions: updateExecutionById(state.executions, id, {
        status: 'error' as const,
        error,
        executionTime: Date.now() - (state.executions.find(e => e.id === id)?.timestamp || Date.now()),
      }),
    }))
  },

  /**
   * Gets a copy of all executions in the queue
   * Returns a mutable copy to prevent accidental store mutations
   *
   * @returns Array of all tool executions
   */
  getQueue: () => {
    return [...get().executions]
  },

  /**
   * Gets all active executions (pending or executing)
   * Useful for determining if tool operations are in progress
   *
   * @returns Array of active executions
   */
  getActive: () => {
    return [...get().executions].filter((exec) => isActiveStatus(exec.status))
  },

  /**
   * Gets a specific execution by ID
   *
   * @param id - The execution ID to look up
   * @returns ToolExecution if found, undefined otherwise
   */
  getById: (id) => {
    return get().executions.find((exec) => exec.id === id)
  },

  /**
   * Finds an execution by its resulting artifact ID
   * Useful for linking artifacts back to their originating execution
   *
   * @param artifactId - The artifact ID to search for
   * @returns ToolExecution if found, undefined otherwise
   */
  findByArtifactId: (artifactId) => {
    return get().executions.find((exec) => exec.artifact?.artifactId === artifactId)
  },

  /**
   * Removes all completed and failed executions from the store
   * Keeps only active executions (pending or executing)
   * Useful for cleaning up execution history
   */
  clearCompleted: () => {
    set((state) => ({
      executions: state.executions.filter((exec) => isActiveStatus(exec.status)),
    }))
  },

  /**
   * Removes a specific execution from the store by ID
   *
   * @param id - The execution ID to remove
   */
  removeExecution: (id) => {
    set((state) => ({
      executions: state.executions.filter((exec) => exec.id !== id),
    }))
  },
}))

/**
 * Selector functions for accessing tool execution store state
 */
export const toolExecutionSelectors = {
  /**
   * Gets an execution by its ID
   *
   * @param state - Tool execution store state
   * @param id - Execution ID to look up
   * @returns ToolExecution if found, undefined otherwise
   */
  getExecutionById: (state: ToolExecutionStore, id: string): ToolExecution | undefined =>
    state.executions.find((exec) => exec.id === id),

  /**
   * Gets all executions with a specific status
   *
   * @param state - Tool execution store state
   * @param status - Status to filter by
   * @returns Array of executions with matching status
   */
  getExecutionsByStatus: (state: ToolExecutionStore, status: ToolExecution['status']): readonly ToolExecution[] =>
    state.executions.filter((exec) => exec.status === status),

  /**
   * Gets all active executions (pending or executing)
   *
   * @param state - Tool execution store state
   * @returns Array of active executions
   */
  getActiveExecutions: (state: ToolExecutionStore): readonly ToolExecution[] =>
    state.executions.filter((exec) => isActiveStatus(exec.status)),

  /**
   * Gets all completed executions
   *
   * @param state - Tool execution store state
   * @returns Array of completed executions
   */
  getCompletedExecutions: (state: ToolExecutionStore): readonly ToolExecution[] =>
    state.executions.filter((exec) => exec.status === 'completed'),

  /**
   * Gets all failed executions
   *
   * @param state - Tool execution store state
   * @returns Array of failed executions
   */
  getFailedExecutions: (state: ToolExecutionStore): readonly ToolExecution[] =>
    state.executions.filter((exec) => exec.status === 'error'),

  /**
   * Gets the total count of executions in the store
   *
   * @param state - Tool execution store state
   * @returns Number of executions
   */
  getExecutionCount: (state: ToolExecutionStore): number => state.executions.length,

  /**
   * Checks if any executions are currently active
   *
   * @param state - Tool execution store state
   * @returns True if at least one execution is active, false otherwise
   */
  hasActiveExecutions: (state: ToolExecutionStore): boolean =>
    state.executions.some((exec) => isActiveStatus(exec.status)),
}
