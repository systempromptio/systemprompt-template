import { create } from 'zustand'
import type { Artifact, ArtifactType } from '@/types/artifact'
import { isPersistedArtifact } from '@/types/artifact'
import { artifactsService } from '@/services/artifacts.service'
import { shouldReplaceItem } from '@/utils/store-helpers'
import { ensureInArray, addToMapping } from './store-utilities'
import { extractAndStoreSkill } from '@/lib/utils/extractArtifactSkills'

type IndexKey = 'byTask' | 'byContext'

interface NormalizeOptions {
  indexKey?: IndexKey
  indexValue?: string
}

interface ArtifactState {
  byId: Record<string, Artifact>
  allIds: readonly string[]
  byContext: Record<string, readonly string[]>
  byTask: Record<string, readonly string[]>
  isLoading: boolean
  error?: string | null
}

function normalizeArtifacts(
  state: ArtifactState,
  artifacts: Artifact[],
  options: NormalizeOptions = {}
): Partial<ArtifactState> {
  const newById = { ...state.byId }
  let newAllIds = [...state.allIds]
  const newByContext = { ...state.byContext }
  const newByTask = { ...state.byTask }
  const artifactIds: string[] = []

  artifacts.forEach((artifact) => {
    newById[artifact.artifactId] = artifact
    artifactIds.push(artifact.artifactId)
    if (!newAllIds.includes(artifact.artifactId)) {
      newAllIds.push(artifact.artifactId)
    }

    if (isPersistedArtifact(artifact)) {
      const contextId = artifact.metadata.context_id
      const taskId = artifact.metadata.task_id

      if (contextId) {
        addToMapping(newByContext, contextId, artifact.artifactId)
      }
      if (taskId) {
        addToMapping(newByTask, taskId, artifact.artifactId)
        if (contextId) {
          extractAndStoreSkill(artifact, contextId, taskId)
        }
      }
    }
  })

  const result: Partial<ArtifactState> = {
    byId: newById,
    allIds: newAllIds,
    byContext: newByContext,
    byTask: newByTask,
    isLoading: false,
  }

  if (options.indexKey && options.indexValue) {
    if (options.indexKey === 'byContext') {
      result.byContext = { ...newByContext, [options.indexValue]: artifactIds }
    } else if (options.indexKey === 'byTask') {
      result.byTask = { ...newByTask, [options.indexValue]: artifactIds }
    }
  }

  return result
}

/**
 * Zustand store interface for managing artifact state
 */
interface ArtifactStore {
  byId: Record<string, Artifact>
  allIds: readonly string[]
  byContext: Readonly<Record<string, readonly string[]>>
  byTask: Readonly<Record<string, readonly string[]>>
  isLoading: boolean
  error: string | null
  selectedArtifactId: string | null
  selectedArtifactIds: readonly string[]
  currentArtifactIndex: number

  fetchAllArtifacts: (authToken: string | null, limit?: number) => Promise<void>
  fetchArtifactsByContext: (contextId: string, authToken: string | null) => Promise<void>
  fetchArtifactsByTask: (taskId: string, authToken: string | null) => Promise<void>
  fetchArtifact: (artifactId: string, authToken: string | null) => Promise<void>
  addArtifact: (artifact: Artifact, taskId?: string, contextId?: string) => void
  clearError: () => void
  getArtifactsByContext: (contextId: string) => Artifact[]
  getArtifactsByTask: (taskId: string) => Artifact[]
  getArtifactsByType: (type: ArtifactType) => Artifact[]
  reset: () => void
  openArtifact: (artifactId: string) => void
  openArtifacts: (artifactIds: string[]) => void
  nextArtifact: () => void
  previousArtifact: () => void
  closeArtifact: () => void
}

/**
 * Zustand store for managing artifacts with normalized state structure
 *
 * State is organized as:
 * - byId: Record mapping artifact IDs to artifact objects
 * - allIds: Array of all artifact IDs
 * - byContext: Record mapping context IDs to arrays of artifact IDs
 * - byTask: Record mapping task IDs to arrays of artifact IDs
 * - isLoading: Loading state indicator
 * - error: Error message if any
 * - selectedArtifactId: Currently selected persisted artifact ID
 *
 * Modal state management:
 * - Multiple artifacts can be navigated via selectedArtifactIds array
 */
export const useArtifactStore = create<ArtifactStore>()((set, get) => ({
  byId: {},
  allIds: [],
  byContext: {},
  byTask: {},
  isLoading: false,
  error: null,
  selectedArtifactId: null,
  selectedArtifactIds: [],
  currentArtifactIndex: 0,

  /**
   * Fetches all artifacts from the API with optional limit
   */
  fetchAllArtifacts: async (authToken, limit?) => {
    set({ isLoading: true, error: null })
    const { artifacts, error } = await artifactsService.listArtifacts(authToken, limit)

    if (error) {
      set({ isLoading: false, error })
      return
    }
    if (artifacts) {
      set((state) => normalizeArtifacts(state, artifacts))
    } else {
      set({ isLoading: false })
    }
  },

  /**
   * Fetches all artifacts for a specific context from the API
   */
  fetchArtifactsByContext: async (contextId, authToken) => {
    set({ isLoading: true, error: null })
    const { artifacts, error } = await artifactsService.listArtifactsByContext(contextId, authToken)

    if (error) {
      set({ isLoading: false, error })
      return
    }
    if (artifacts) {
      set((state) => normalizeArtifacts(state, artifacts, { indexKey: 'byContext', indexValue: contextId }))
    } else {
      set({ isLoading: false })
    }
  },

  /**
   * Fetches all artifacts for a specific task from the API
   */
  fetchArtifactsByTask: async (taskId, authToken) => {
    set({ isLoading: true, error: null })
    const { artifacts, error } = await artifactsService.listArtifactsByTask(taskId, authToken)

    if (error) {
      set({ isLoading: false, error })
      return
    }
    if (artifacts) {
      set((state) => normalizeArtifacts(state, artifacts, { indexKey: 'byTask', indexValue: taskId }))
    } else {
      set({ isLoading: false })
    }
  },

  /**
   * Fetches a single artifact by ID from the API
   */
  fetchArtifact: async (artifactId, authToken) => {
    set({ isLoading: true, error: null })
    const { artifact, error } = await artifactsService.getArtifact(artifactId, authToken)

    if (error) {
      set({ isLoading: false, error })
      return
    }
    if (artifact) {
      set((state) => normalizeArtifacts(state, [artifact]))
    } else {
      set({ isLoading: false })
    }
  },

  /**
   * Adds or updates an artifact in the store
   * Uses shouldReplaceItem to determine if update should proceed based on metadata timestamps
   *
   * @param artifact - The artifact to add or update
   * @param taskId - Optional task ID to associate the artifact with
   * @param contextId - Optional context ID to associate the artifact with
   */
  addArtifact: (artifact, taskId?, contextId?) => {
    set((state) => {
      const existing = state.byId[artifact.artifactId]

      if (!shouldReplaceItem(artifact.metadata, existing?.metadata)) {
        return state
      }

      const newByContext = { ...state.byContext }
      const newByTask = { ...state.byTask }

      if (contextId) {
        addToMapping(newByContext, contextId, artifact.artifactId)
      }

      if (taskId) {
        addToMapping(newByTask, taskId, artifact.artifactId)
      }

      return {
        byId: { ...state.byId, [artifact.artifactId]: artifact },
        allIds: ensureInArray(artifact.artifactId, state.allIds),
        byContext: newByContext,
        byTask: newByTask,
      }
    })
  },

  /**
   * Clears any error state
   */
  clearError: () => set({ error: null }),

  /**
   * Gets all artifacts for a specific context
   *
   * @param contextId - The context ID to get artifacts for
   * @returns Array of artifacts belonging to the context
   */
  getArtifactsByContext: (contextId) => {
    const state = get()
    const artifactIds = state.byContext[contextId] || []
    return artifactIds
      .map((id) => state.byId[id])
      .filter((artifact): artifact is Artifact => artifact !== undefined)
  },

  /**
   * Gets all artifacts for a specific task
   *
   * @param taskId - The task ID to get artifacts for
   * @returns Array of artifacts belonging to the task
   */
  getArtifactsByTask: (taskId) => {
    const state = get()
    const artifactIds = state.byTask[taskId] || []
    return artifactIds
      .map((id) => state.byId[id])
      .filter((artifact): artifact is Artifact => artifact !== undefined)
  },

  /**
   * Gets all artifacts of a specific type
   *
   * @param type - The artifact type to filter by
   * @returns Array of artifacts with matching type
   */
  getArtifactsByType: (type) => {
    const state = get()
    return state.allIds
      .map(id => state.byId[id])
      .filter((artifact) => artifact.metadata.artifact_type === type)
  },

  /**
   * Resets the entire store to initial state
   */
  reset: () => {
    set({
      byId: {},
      allIds: [],
      byContext: {},
      byTask: {},
      isLoading: false,
      error: null,
      selectedArtifactId: null,
      selectedArtifactIds: [],
      currentArtifactIndex: 0,
    })
  },

  /**
   * Opens a persisted artifact in the modal
   *
   * @param artifactId - The artifact ID to open
   */
  openArtifact: (artifactId: string) => {
    set({
      selectedArtifactId: artifactId,
      selectedArtifactIds: [artifactId],
      currentArtifactIndex: 0,
    })
  },

  /**
   * Opens multiple artifacts for navigation
   * Sets the first artifact as the current selection
   *
   * @param artifactIds - Array of artifact IDs to open for navigation
   */
  openArtifacts: (artifactIds: string[]) => {
    if (artifactIds.length === 0) return
    set({
      selectedArtifactId: artifactIds[0],
      selectedArtifactIds: artifactIds,
      currentArtifactIndex: 0,
    })
  },

  /**
   * Navigates to the next artifact in the selectedArtifactIds array
   */
  nextArtifact: () => {
    const state = get()
    if (state.selectedArtifactIds.length === 0) return
    const nextIndex = (state.currentArtifactIndex + 1) % state.selectedArtifactIds.length
    const nextArtifactId = state.selectedArtifactIds[nextIndex]
    set({
      selectedArtifactId: nextArtifactId,
      currentArtifactIndex: nextIndex,
    })
  },

  /**
   * Navigates to the previous artifact in the selectedArtifactIds array
   */
  previousArtifact: () => {
    const state = get()
    if (state.selectedArtifactIds.length === 0) return
    const prevIndex = (state.currentArtifactIndex - 1 + state.selectedArtifactIds.length) % state.selectedArtifactIds.length
    const prevArtifactId = state.selectedArtifactIds[prevIndex]
    set({
      selectedArtifactId: prevArtifactId,
      currentArtifactIndex: prevIndex,
    })
  },

  /**
   * Closes any open artifact
   */
  closeArtifact: () => {
    set({
      selectedArtifactId: null,
      selectedArtifactIds: [],
      currentArtifactIndex: 0,
    })
  },
}))

/**
 * Selector functions for accessing artifact store state
 */
export const artifactSelectors = {
  /**
   * Gets an artifact by its ID
   *
   * @param state - Artifact store state
   * @param id - Artifact ID to look up
   * @returns Artifact object if found, undefined otherwise
   */
  getArtifactById: (state: ArtifactStore, id: string): Artifact | undefined =>
    state.byId[id],

  /**
   * Gets the currently selected persisted artifact
   *
   * @param state - Artifact store state
   * @returns Currently selected artifact or null
   */
  getSelectedArtifact: (state: ArtifactStore): Artifact | null => {
    const { selectedArtifactId, byId } = state
    return selectedArtifactId && byId[selectedArtifactId] ? byId[selectedArtifactId] : null
  },

  /**
   * Gets all artifact IDs for a specific context
   *
   * @param state - Artifact store state
   * @param contextId - Context ID to get artifact IDs for
   * @returns Array of artifact IDs for the context
   */
  getArtifactsByContextIds: (state: ArtifactStore, contextId: string): readonly string[] =>
    state.byContext[contextId] ?? ([] as const),

  /**
   * Gets all artifact IDs for a specific task
   *
   * @param state - Artifact store state
   * @param taskId - Task ID to get artifact IDs for
   * @returns Array of artifact IDs for the task
   */
  getArtifactsByTaskIds: (state: ArtifactStore, taskId: string): readonly string[] =>
    state.byTask[taskId] ?? ([] as const),

  /**
   * Gets the total count of artifacts in the store
   *
   * @param state - Artifact store state
   * @returns Number of artifacts
   */
  getArtifactCount: (state: ArtifactStore): number => state.allIds.length,

  /**
   * Checks if artifacts are currently being loaded
   *
   * @param state - Artifact store state
   * @returns True if loading, false otherwise
   */
  isLoading: (state: ArtifactStore): boolean => state.isLoading,

  /**
   * Checks if there is an error state
   *
   * @param state - Artifact store state
   * @returns True if error exists, false otherwise
   */
  hasError: (state: ArtifactStore): boolean => state.error !== null,

  /**
   * Gets the current error message
   *
   * @param state - Artifact store state
   * @returns Error message or null
   */
  getError: (state: ArtifactStore): string | null => state.error,
}
