import { create } from 'zustand'
import { persist } from 'zustand/middleware'

/**
 * User preference settings persisted to local storage.
 */
export interface SettingsStore {
  debugMode: boolean
  setDebugMode: (enabled: boolean) => void
  leftSidebarVisible: boolean
  toggleLeftSidebar: () => void
}

/**
 * Store for application user preferences.
 * Settings are automatically persisted to local storage.
 */
export const useSettingsStore = create<SettingsStore>()(
  persist(
    (set) => ({
      debugMode: false,

      /**
       * Enable or disable debug mode.
       * @param enabled - Debug mode flag
       */
      setDebugMode: (enabled) => set({ debugMode: enabled }),

      leftSidebarVisible: false,

      /**
       * Toggle left sidebar visibility.
       */
      toggleLeftSidebar: () => set((state) => ({ leftSidebarVisible: !state.leftSidebarVisible })),
    }),
    {
      name: 'systemprompt-settings-v2',
    }
  )
)

/**
 * Selectors for reading settings state.
 */
export const settingsSelectors = {
  /**
   * Check if debug mode is enabled.
   * @param state - Current settings state
   * @returns True if debug mode enabled
   */
  isDebugModeEnabled: (state: SettingsStore): boolean => state.debugMode,

  /**
   * Check if left sidebar is visible.
   * @param state - Current settings state
   * @returns True if sidebar visible
   */
  isLeftSidebarVisible: (state: SettingsStore): boolean => state.leftSidebarVisible,
}
