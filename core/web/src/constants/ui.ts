/**
 * UI State Keys - Keys for storing UI state in storage and stores
 */
export const UIStateKey = {
  AUTH_STORAGE: 'auth-storage',
  CHAT_STORAGE: 'chat-storage',
  CONTEXT_STORAGE: 'context-storage',
  THEME: 'theme',
  SIDEBAR_COLLAPSED: 'sidebar-collapsed',
  FIRST_VISIT: 'first-visit',
} as const

export type UIStateKey = typeof UIStateKey[keyof typeof UIStateKey]

/**
 * Theme Names - Available color themes
 */
export const Theme = {
  GRADIENT: 'gradient',
  ACCENT: 'accent',
  GLASS: 'glass',
  MINIMAL: 'minimal',
  DEFAULT: 'default',
} as const

export type Theme = typeof Theme[keyof typeof Theme]

/**
 * Dialog/Modal Types - Different kinds of UI dialogs
 */
export const DialogType = {
  TOOL_RESULT: 'tool_result',
  AUTH: 'auth',
  CONFIRMATION: 'confirmation',
  ERROR: 'error',
  INFO: 'info',
} as const

export type DialogType = typeof DialogType[keyof typeof DialogType]

/**
 * Animation Classes - Tailwind animation class names
 */
export const Animation = {
  SLIDE_IN_UP: 'animate-slideInUp',
  FADE_IN: 'animate-fadeIn',
  SPIN: 'animate-spin',
  PULSE: 'animate-pulse',
} as const

export type Animation = typeof Animation[keyof typeof Animation]
