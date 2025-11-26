/**
 * Zustand store utility functions
 * Common helpers for state management across stores
 */

/**
 * Generate a storage key for localStorage, optionally scoped to a user
 */
export const getStorageKey = (key: string, userId?: string): string => {
  return userId ? `${key}:${userId}` : key
}

/**
 * Load a value from localStorage
 */
export const loadFromStorage = (key: string): string | undefined => {
  if (typeof window === 'undefined') return undefined
  try {
    const value = localStorage.getItem(key)
    return value ?? undefined
  } catch {
    return undefined
  }
}

/**
 * Save a value to localStorage
 */
export const saveToStorage = (key: string, value: string): void => {
  if (typeof window === 'undefined') return
  try {
    localStorage.setItem(key, value)
  } catch {
    // Silently fail if localStorage is unavailable
  }
}

/**
 * Create a partial state update for setting an error
 */
export const setError = (error: string) => ({
  error,
})

/**
 * Create a partial state update for clearing an error
 */
export const clearError = () => ({
  error: null,
})

/**
 * Ensures an item is in an array, returns new array if modified
 */
export const ensureInArray = <T,>(item: T, array: readonly T[]): readonly T[] => {
  if (array.includes(item)) {
    return array
  }
  return [...array, item]
}

/**
 * Add a value to an array within a record (mapping)
 * Creates the array if it doesn't exist, ensures uniqueness
 */
export const addToMapping = <K extends string | number | symbol>(
  mapping: Record<K, readonly any[]>,
  key: K,
  value: any
): void => {
  if (!mapping[key]) {
    mapping[key] = [value]
  } else if (!mapping[key].includes(value)) {
    mapping[key] = [...mapping[key], value]
  }
}

/**
 * Deep clone a record containing arrays
 * Used to create immutable copies of nested data structures
 */
export const cloneRecordArrays = <K extends string | number | symbol, V>(
  record: Readonly<Record<K, readonly V[]>>
): Record<K, readonly V[]> => {
  const cloned: Record<K, readonly V[]> = {} as Record<K, readonly V[]>
  for (const key in record) {
    cloned[key] = [...record[key]]
  }
  return cloned
}

/**
 * Open a persisted item and close any ephemeral item
 * Returns modal state updates
 */
export const openPersisted = <T,>(selectedId: string) => ({
  selectedId,
  ephemeralItem: null as T | null,
})

/**
 * Open an ephemeral item and close any persisted selection
 * Returns modal state updates
 */
export const openEphemeral = <T,>(item: T) => ({
  selectedId: null as string | null,
  ephemeralItem: item,
})

/**
 * Close any open artifact (persisted or ephemeral)
 * Returns modal state updates
 */
export const closeModal = <T,>() => ({
  selectedId: null as string | null,
  ephemeralItem: null as T | null,
})
