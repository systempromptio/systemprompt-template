import { create } from 'zustand'

/**
 * Primary view types available in the application.
 */
export type ViewType = 'conversation' | 'tasks' | 'artifacts'

/**
 * UI view state for tracking active section.
 */
interface ViewStore {
  activeView: ViewType
  setActiveView: (view: ViewType) => void
}

/**
 * Store for managing primary UI view state.
 */
export const useViewStore = create<ViewStore>()((set) => ({
  activeView: 'conversation',

  /**
   * Set the active view section.
   * @param view - View type to activate
   */
  setActiveView: (view) => set({ activeView: view }),
}))

/**
 * Selectors for reading view state.
 */
export const viewSelectors = {
  /**
   * Get currently active view type.
   * @param state - Current view state
   * @returns Active view type
   */
  getActiveView: (state: ViewStore): ViewType => state.activeView,

  /**
   * Check if conversation view is active.
   * @param state - Current view state
   * @returns True if conversation view
   */
  isConversationView: (state: ViewStore): boolean => state.activeView === 'conversation',

  /**
   * Check if tasks view is active.
   * @param state - Current view state
   * @returns True if tasks view
   */
  isTasksView: (state: ViewStore): boolean => state.activeView === 'tasks',

  /**
   * Check if artifacts view is active.
   * @param state - Current view state
   * @returns True if artifacts view
   */
  isArtifactsView: (state: ViewStore): boolean => state.activeView === 'artifacts',
}
