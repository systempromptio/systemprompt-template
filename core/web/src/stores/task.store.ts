import { create } from 'zustand'
import type { TaskState } from '@a2a-js/sdk'
import type { Task } from '@/types/task'
import { tasksService } from '@/services/tasks.service'
import { shouldReplaceItem } from '@/utils/store-helpers'

/**
 * Helper function to ensure task ID exists in array
 *
 * @param taskId - The task ID to check
 * @param allIds - Current array of task IDs
 * @returns Updated array with task ID included if it wasn't already present
 */
const ensureTaskIdInArray = (taskId: string, allIds: readonly string[]): readonly string[] => {
  return allIds.includes(taskId) ? allIds : [...allIds, taskId]
}

/**
 * Helper function to update task normalized state with new tasks
 *
 * @param state - Current store state
 * @param tasks - Array of tasks to add/update
 * @returns Updated normalized state
 */
const normalizeTasksIntoState = (
  state: { byId: Record<string, Task>; allIds: readonly string[] },
  tasks: Task[]
) => {
  const newById = { ...state.byId }
  const newAllIds = [...state.allIds]

  tasks.forEach((task) => {
    newById[task.id] = task
    if (!newAllIds.includes(task.id)) {
      newAllIds.push(task.id)
    }
  })

  return { newById, newAllIds }
}

/**
 * Zustand store interface for managing task state
 */
interface TaskStore {
  byId: Record<string, Task>
  allIds: readonly string[]
  byContext: Readonly<Record<string, readonly string[]>>
  isLoading: boolean
  error: string | null

  fetchTasksByContext: (contextId: string, authToken: string | null) => Promise<void>
  fetchTask: (taskId: string, authToken: string | null) => Promise<void>
  fetchTasks: (authToken: string | null, status?: string, limit?: number) => Promise<void>
  updateTask: (task: Task) => void
  clearError: () => void
  getTasksByContext: (contextId: string) => Task[]
  getTasksByStatus: (status: TaskState) => Task[]
  reset: () => void
}

/**
 * Zustand store for managing tasks with normalized state structure
 *
 * State is organized as:
 * - byId: Record mapping task IDs to task objects
 * - allIds: Array of all task IDs
 * - byContext: Record mapping context IDs to arrays of task IDs
 * - isLoading: Loading state indicator
 * - error: Error message if any
 *
 * This normalized structure enables efficient lookups and updates
 */
export const useTaskStore = create<TaskStore>()((set, get) => ({
  byId: {},
  allIds: [],
  byContext: {},
  isLoading: false,
  error: null,

  /**
   * Fetches all tasks for a specific context from the API
   *
   * @param contextId - The context ID to fetch tasks for
   * @param authToken - Authentication token (optional)
   * @returns Promise that resolves when fetch is complete
   */
  fetchTasksByContext: async (contextId, authToken) => {
    set({ isLoading: true, error: null })

    const { tasks, error } = await tasksService.listTasksByContext(contextId, authToken)

    if (error) {
      set({ isLoading: false, error })
      return
    }

    if (tasks) {
      set((state) => {
        const { newById, newAllIds } = normalizeTasksIntoState(state, tasks)
        const taskIds = tasks.map(task => task.id)

        return {
          byId: newById,
          allIds: newAllIds,
          byContext: { ...state.byContext, [contextId]: taskIds },
          isLoading: false,
        }
      })
    } else {
      set({ isLoading: false })
    }
  },

  /**
   * Fetches a single task by ID from the API
   *
   * @param taskId - The task ID to fetch
   * @param authToken - Authentication token (optional)
   * @returns Promise that resolves when fetch is complete
   */
  fetchTask: async (taskId, authToken) => {
    set({ isLoading: true, error: null })

    const { task, error } = await tasksService.getTask(taskId, authToken)

    if (error) {
      set({ isLoading: false, error })
      return
    }

    if (task) {
      set((state) => ({
        byId: { ...state.byId, [task.id]: task },
        allIds: ensureTaskIdInArray(task.id, state.allIds),
        isLoading: false,
      }))
    } else {
      set({ isLoading: false })
    }
  },

  /**
   * Fetches all tasks matching optional filter criteria
   *
   * @param authToken - Authentication token (optional)
   * @param status - Optional status filter
   * @param limit - Optional result limit
   * @returns Promise that resolves when fetch is complete
   */
  fetchTasks: async (authToken, status?, limit?) => {
    set({ isLoading: true, error: null })

    const { tasks, error } = await tasksService.listTasks(authToken, status, limit)

    if (error) {
      set({ isLoading: false, error })
      return
    }

    if (tasks) {
      set((state) => {
        const { newById, newAllIds } = normalizeTasksIntoState(state, tasks)

        return {
          byId: newById,
          allIds: newAllIds,
          isLoading: false,
        }
      })
    } else {
      set({ isLoading: false })
    }
  },

  /**
   * Updates or adds a task to the store
   * Uses shouldReplaceItem to determine if update should proceed based on metadata timestamps
   *
   * @param task - The task to update or add
   */
  updateTask: (task) => {
    set((state) => {
      const existing = state.byId[task.id]

      console.log('%c[TASK_STORE] updateTask called', 'background: #9966ff; color: white;', {
        timestamp: new Date().toISOString(),
        taskId: task.id,
        isNew: !existing,
        incomingStatus: task.status?.state,
        existingStatus: existing?.status?.state,
        incomingHistoryLength: task.history?.length || 0,
        existingHistoryLength: existing?.history?.length || 0,
        incomingHistoryRoles: task.history?.map(m => m.role),
        existingHistoryRoles: existing?.history?.map(m => m.role),
        incomingHistoryTexts: task.history?.map(m => ({
          role: m.role,
          textPreview: m.parts?.[0]?.kind === 'text' ? (m.parts[0] as { text?: string }).text?.substring(0, 50) : 'N/A'
        }))
      })

      if (!shouldReplaceItem(task.metadata, existing?.metadata)) {
        console.log('%c[TASK_STORE] Skipping update - existing task is newer', 'background: #ff0000; color: white;', {
          timestamp: new Date().toISOString(),
          taskId: task.id,
          existingMetadata: existing?.metadata,
          newMetadata: task.metadata
        })
        return state
      }

      const isNew = !existing
      const userMessage = task.history?.[0]?.parts?.[0]
      console.log(`[TASK_STORE] ${isNew ? 'Adding new' : 'Updating'} task`, {
        timestamp: new Date().toISOString(),
        taskId: task.id,
        contextId: task.contextId,
        status: task.status?.state,
        historyLength: task.history?.length || 0,
        userMessagePreview: userMessage?.kind === 'text' ? (userMessage as { text?: string }).text?.substring(0, 50) : 'N/A'
      })

      if (isNew && (!task.history || task.history.length === 0)) {
        console.warn(
          '%c[TASK_STORE] WARNING: Task created with EMPTY HISTORY! User message will not display until task completion.',
          'background: #ff0000; color: #ffffff; font-size: 14px; font-weight: bold; padding: 4px 8px;',
          {
            taskId: task.id,
            status: task.status?.state,
            fullTask: task
          }
        )
      }

      const newByContext = { ...state.byContext }
      const existingTaskIds = state.byContext[task.contextId] || []
      if (!existingTaskIds.includes(task.id)) {
        newByContext[task.contextId] = [...existingTaskIds, task.id]
      }

      const mergedTask = {
        ...task,
        history: (task.history?.length ?? 0) > 0
          ? task.history
          : existing?.history ?? []
      }

      return {
        byId: { ...state.byId, [task.id]: mergedTask },
        allIds: ensureTaskIdInArray(task.id, state.allIds),
        byContext: newByContext,
      }
    })
  },

  /**
   * Clears any error state
   */
  clearError: () => set({ error: null }),

  /**
   * Gets all tasks for a specific context
   *
   * @param contextId - The context ID to get tasks for
   * @returns Array of tasks belonging to the context
   */
  getTasksByContext: (contextId) => {
    const state = get()
    const taskIds = state.byContext[contextId] || []
    return taskIds
      .map((id) => state.byId[id])
      .filter((task): task is Task => task !== undefined)
  },

  /**
   * Gets all tasks with a specific status
   *
   * @param status - The task status to filter by
   * @returns Array of tasks with matching status
   */
  getTasksByStatus: (status) => {
    const state = get()
    return state.allIds
      .map(id => state.byId[id])
      .filter((task) => task.status.state === status)
  },

  /**
   * Resets the entire store to initial state
   */
  reset: () => {
    set({
      byId: {},
      allIds: [],
      byContext: {},
      isLoading: false,
      error: null,
    })
  },
}))

/**
 * Selector functions for accessing task store state
 */
export const taskSelectors = {
  /**
   * Gets a task by its ID
   *
   * @param state - Task store state
   * @param id - Task ID to look up
   * @returns Task object if found, undefined otherwise
   */
  getTaskById: (state: TaskStore, id: string): Task | undefined =>
    state.byId[id],

  /**
   * Gets all task IDs for a specific context
   *
   * @param state - Task store state
   * @param contextId - Context ID to get task IDs for
   * @returns Array of task IDs for the context
   */
  getTasksByContextIds: (state: TaskStore, contextId: string): readonly string[] =>
    state.byContext[contextId] ?? ([] as const),

  /**
   * Gets the total count of tasks in the store
   *
   * @param state - Task store state
   * @returns Number of tasks
   */
  getTaskCount: (state: TaskStore): number => state.allIds.length,

  /**
   * Checks if tasks are currently being loaded
   *
   * @param state - Task store state
   * @returns True if loading, false otherwise
   */
  isLoading: (state: TaskStore): boolean => state.isLoading,

  /**
   * Checks if there is an error state
   *
   * @param state - Task store state
   * @returns True if error exists, false otherwise
   */
  hasError: (state: TaskStore): boolean => state.error !== null,

  /**
   * Gets the current error message
   *
   * @param state - Task store state
   * @returns Error message or null
   */
  getError: (state: TaskStore): string | null => state.error ?? null,

  /**
   * Checks if any tasks exist in the store
   *
   * @param state - Task store state
   * @returns True if at least one task exists, false otherwise
   */
  hasAnyTasks: (state: TaskStore): boolean => state.allIds.length > 0,
}
