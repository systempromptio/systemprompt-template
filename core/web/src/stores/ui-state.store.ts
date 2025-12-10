import { create } from 'zustand'
import type { ExecutionStep } from '@/types/execution'
import type { EphemeralArtifact } from '@/types/artifact'

export interface ToolExecution {
  id: string
  toolName: string
  serverName: string
  status: 'pending' | 'executing' | 'completed' | 'error'
  artifactId?: string
  error?: string
  timestamp: number
  parameters?: Record<string, unknown>
  executionTime?: number
}

export interface InputRequest {
  taskId: string
  messageId: string
  message?: unknown
  timestamp: Date
}

export interface AuthRequest {
  taskId: string
  messageId: string
  message?: unknown
  timestamp: Date
}

interface UIStateStore {
  activeStreamingTaskId: string | null

  stepsById: Record<string, ExecutionStep>
  stepIdsByTask: Record<string, readonly string[]>
  stepIdsByContext: Record<string, readonly string[]>

  toolExecutionsById: Record<string, ToolExecution>
  toolExecutionIdsByTask: Record<string, readonly string[]>

  inputRequestsByTask: Record<string, InputRequest>
  authRequestsByTask: Record<string, AuthRequest>

  ephemeralArtifact: EphemeralArtifact | null

  setStreaming: (taskId: string | null) => void

  addStep: (step: ExecutionStep, contextId?: string) => void
  addSteps: (steps: ExecutionStep[], contextId?: string) => void
  updateStep: (stepId: string, updates: Partial<ExecutionStep>) => void
  getStepsByTask: (taskId: string) => ExecutionStep[]
  getStepsByContext: (contextId: string) => ExecutionStep[]
  clearStepsByTask: (taskId: string) => void

  addToolExecution: (taskId: string, execution: ToolExecution) => void
  completeToolExecution: (executionId: string, artifactId?: string) => void
  failToolExecution: (executionId: string, error: string) => void
  removeToolExecution: (executionId: string) => void
  getAllToolExecutions: () => ToolExecution[]
  getToolExecutionsByTask: (taskId: string) => ToolExecution[]
  getActiveToolExecutions: () => ToolExecution[]
  findToolExecutionByArtifactId: (artifactId: string) => ToolExecution | undefined
  getToolExecutionById: (executionId: string) => ToolExecution | undefined

  registerInputRequest: (request: InputRequest) => void
  resolveInputRequest: (taskId: string) => void
  registerAuthRequest: (request: AuthRequest) => void
  resolveAuthRequest: (taskId: string) => void
  getFirstPendingInputRequest: () => InputRequest | undefined
  getFirstPendingAuthRequest: () => AuthRequest | undefined
  hasPendingInputRequests: () => boolean
  hasPendingAuthRequests: () => boolean

  setEphemeralArtifact: (artifact: EphemeralArtifact | null) => void

  reset: () => void
  resetForContext: (contextId: string) => void
}

const initialState = {
  activeStreamingTaskId: null,
  stepsById: {},
  stepIdsByTask: {},
  stepIdsByContext: {},
  toolExecutionsById: {},
  toolExecutionIdsByTask: {},
  inputRequestsByTask: {},
  authRequestsByTask: {},
  ephemeralArtifact: null,
}

export const useUIStateStore = create<UIStateStore>()((set, get) => ({
  ...initialState,

  setStreaming: (taskId) => set({ activeStreamingTaskId: taskId }),

  addStep: (step, contextId) => {
    set((state) => {
      const existing = state.stepsById[step.stepId]

      if (existing) {
        return {
          stepsById: {
            ...state.stepsById,
            [step.stepId]: {
              ...existing,
              ...step,
              errorMessage: step.errorMessage ?? existing.errorMessage,
              durationMs: step.durationMs ?? existing.durationMs,
              content: step.content ?? existing.content,
            },
          },
        }
      }

      const taskSteps = state.stepIdsByTask[step.taskId] || []
      const newStepIdsByTask = taskSteps.includes(step.stepId)
        ? state.stepIdsByTask
        : { ...state.stepIdsByTask, [step.taskId]: [...taskSteps, step.stepId] }

      const newStepIdsByContext = contextId
        ? (() => {
            const contextSteps = state.stepIdsByContext[contextId] || []
            return contextSteps.includes(step.stepId)
              ? state.stepIdsByContext
              : { ...state.stepIdsByContext, [contextId]: [...contextSteps, step.stepId] }
          })()
        : state.stepIdsByContext

      return {
        stepsById: { ...state.stepsById, [step.stepId]: step },
        stepIdsByTask: newStepIdsByTask,
        stepIdsByContext: newStepIdsByContext,
      }
    })
  },

  addSteps: (steps, contextId) => {
    steps.forEach((step) => get().addStep(step, contextId))
  },

  updateStep: (stepId, updates) => {
    set((state) => {
      const existing = state.stepsById[stepId]
      if (!existing) return state
      return {
        stepsById: {
          ...state.stepsById,
          [stepId]: { ...existing, ...updates },
        },
      }
    })
  },

  getStepsByTask: (taskId) => {
    const state = get()
    const stepIds = state.stepIdsByTask[taskId] || []
    return stepIds
      .map((id) => state.stepsById[id])
      .filter((step): step is ExecutionStep => !!step)
      .sort((a, b) => new Date(a.startedAt).getTime() - new Date(b.startedAt).getTime())
  },

  getStepsByContext: (contextId) => {
    const state = get()
    const stepIds = state.stepIdsByContext[contextId] || []
    return stepIds
      .map((id) => state.stepsById[id])
      .filter((step): step is ExecutionStep => !!step)
      .sort((a, b) => new Date(a.startedAt).getTime() - new Date(b.startedAt).getTime())
  },

  clearStepsByTask: (taskId) => {
    set((state) => {
      const stepIds = state.stepIdsByTask[taskId] || []
      const newStepsById = { ...state.stepsById }
      stepIds.forEach((id) => delete newStepsById[id])
      const { [taskId]: _, ...newStepIdsByTask } = state.stepIdsByTask
      return { stepsById: newStepsById, stepIdsByTask: newStepIdsByTask }
    })
  },

  addToolExecution: (taskId, execution) => {
    set((state) => {
      const taskExecutions = state.toolExecutionIdsByTask[taskId] || []
      return {
        toolExecutionsById: {
          ...state.toolExecutionsById,
          [execution.id]: execution,
        },
        toolExecutionIdsByTask: {
          ...state.toolExecutionIdsByTask,
          [taskId]: [...taskExecutions, execution.id],
        },
      }
    })
  },

  completeToolExecution: (executionId, artifactId) => {
    set((state) => {
      const execution = state.toolExecutionsById[executionId]
      if (!execution) return state
      return {
        toolExecutionsById: {
          ...state.toolExecutionsById,
          [executionId]: {
            ...execution,
            status: 'completed' as const,
            artifactId,
            executionTime: Date.now() - execution.timestamp,
          },
        },
      }
    })
  },

  failToolExecution: (executionId, error) => {
    set((state) => {
      const execution = state.toolExecutionsById[executionId]
      if (!execution) return state
      return {
        toolExecutionsById: {
          ...state.toolExecutionsById,
          [executionId]: {
            ...execution,
            status: 'error' as const,
            error,
            executionTime: Date.now() - execution.timestamp,
          },
        },
      }
    })
  },

  getToolExecutionsByTask: (taskId) => {
    const state = get()
    const executionIds = state.toolExecutionIdsByTask[taskId] || []
    return executionIds
      .map((id) => state.toolExecutionsById[id])
      .filter((exec): exec is ToolExecution => !!exec)
  },

  getActiveToolExecutions: () => {
    const state = get()
    return Object.values(state.toolExecutionsById).filter(
      (exec) => exec.status === 'pending' || exec.status === 'executing'
    )
  },

  findToolExecutionByArtifactId: (artifactId) => {
    const state = get()
    return Object.values(state.toolExecutionsById).find(
      (exec) => exec.artifactId === artifactId
    )
  },

  getToolExecutionById: (executionId) => {
    return get().toolExecutionsById[executionId]
  },

  removeToolExecution: (executionId) => {
    set((state) => {
      const { [executionId]: removed, ...remainingExecutions } = state.toolExecutionsById
      if (!removed) return state

      const newToolExecutionIdsByTask = { ...state.toolExecutionIdsByTask }
      for (const taskId of Object.keys(newToolExecutionIdsByTask)) {
        newToolExecutionIdsByTask[taskId] = newToolExecutionIdsByTask[taskId].filter(
          (id) => id !== executionId
        )
      }

      return {
        toolExecutionsById: remainingExecutions,
        toolExecutionIdsByTask: newToolExecutionIdsByTask,
      }
    })
  },

  getAllToolExecutions: () => {
    return Object.values(get().toolExecutionsById)
  },

  registerInputRequest: (request) => {
    set((state) => ({
      inputRequestsByTask: {
        ...state.inputRequestsByTask,
        [request.taskId]: request,
      },
    }))
  },

  resolveInputRequest: (taskId) => {
    set((state) => {
      const { [taskId]: _, ...rest } = state.inputRequestsByTask
      return { inputRequestsByTask: rest }
    })
  },

  registerAuthRequest: (request) => {
    set((state) => ({
      authRequestsByTask: {
        ...state.authRequestsByTask,
        [request.taskId]: request,
      },
    }))
  },

  resolveAuthRequest: (taskId) => {
    set((state) => {
      const { [taskId]: _, ...rest } = state.authRequestsByTask
      return { authRequestsByTask: rest }
    })
  },

  getFirstPendingInputRequest: () => {
    const requests = Object.values(get().inputRequestsByTask)
    return requests.sort((a, b) => a.timestamp.getTime() - b.timestamp.getTime())[0]
  },

  getFirstPendingAuthRequest: () => {
    const requests = Object.values(get().authRequestsByTask)
    return requests.sort((a, b) => a.timestamp.getTime() - b.timestamp.getTime())[0]
  },

  hasPendingInputRequests: () => Object.keys(get().inputRequestsByTask).length > 0,
  hasPendingAuthRequests: () => Object.keys(get().authRequestsByTask).length > 0,

  setEphemeralArtifact: (artifact) => set({ ephemeralArtifact: artifact }),

  reset: () => set(initialState),

  resetForContext: (contextId) => {
    set((state) => {
      const stepIds = state.stepIdsByContext[contextId] || []
      const newStepsById = { ...state.stepsById }
      stepIds.forEach((id) => delete newStepsById[id])
      const { [contextId]: _, ...newStepIdsByContext } = state.stepIdsByContext

      return {
        stepsById: newStepsById,
        stepIdsByContext: newStepIdsByContext,
      }
    })
  },
}))

export const uiStateSelectors = {
  isTaskStreaming: (state: UIStateStore, taskId: string): boolean =>
    state.activeStreamingTaskId === taskId,

  hasStepsForTask: (state: UIStateStore, taskId: string): boolean =>
    (state.stepIdsByTask[taskId]?.length ?? 0) > 0,

  getStepCount: (state: UIStateStore, taskId: string): number =>
    state.stepIdsByTask[taskId]?.length ?? 0,
}
