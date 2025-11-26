/**
 * Common type definitions for React hooks across the application.
 *
 * Centralizes hook interface definitions to promote DRY principles and
 * ensure consistency across the codebase. All hooks that return similar
 * state patterns should use types from this module.
 *
 * @module types/hooks
 */

/**
 * Standard async hook state pattern used by data-fetching hooks.
 *
 * Used by hooks that fetch data asynchronously and track loading/error states.
 * Generic parameter T represents the type of data being fetched.
 *
 * @template T The type of data being managed by the hook
 *
 * @example
 * ```typescript
 * const { data, loading, error } = useMyData<User>()
 * if (data) console.log(data.id)
 * ```
 */
export interface AsyncHookState<T> {
  /**
   * The fetched data, or null if not yet loaded or load failed
   */
  data: T | null

  /**
   * Whether data is currently being fetched
   */
  loading: boolean

  /**
   * Error message if fetch failed, null otherwise
   */
  error: string | null
}

/**
 * Standard connection hook state pattern for stream/socket connections.
 *
 * Used by hooks that manage long-lived connections (SSE, WebSocket, etc.).
 * Tracks connection status, connection state, and error conditions.
 *
 * @example
 * ```typescript
 * const { isConnected, isConnecting, error } = useSSEConnection(options)
 * if (isConnected) sendData()
 * ```
 */
export interface ConnectionHookState {
  /**
   * Whether currently connected to the server
   */
  isConnected: boolean

  /**
   * Whether connection is currently being established
   */
  isConnecting: boolean

  /**
   * Error from last connection attempt, null if no error
   */
  error: Error | null
}

/**
 * Standard authentication hook return type.
 *
 * Used by auth hooks that manage user authentication state and provide
 * auth-related operations like login, logout, permission checks.
 *
 * @example
 * ```typescript
 * const { isAuthenticated, requireAuth, logout } = useAuth()
 * if (!isAuthenticated) requireAuth()
 * ```
 */
export interface AuthHookState {
  /**
   * Whether user is currently authenticated with valid token
   */
  isAuthenticated: boolean

  /**
   * Current user's email address, null if not authenticated
   */
  email: string | null

  /**
   * Current user's username, null if not authenticated
   */
  username: string | null

  /**
   * Array of permission scopes (e.g., 'admin', 'user')
   */
  scopes: string[]

  /**
   * User type: 'real', 'anon', or null if not determined
   */
  userType: string | null
}

/**
 * Standard execution hook state for operations that track progress.
 *
 * Used by hooks that execute operations and track their progress/completion.
 * Generic parameter T represents the result type of execution.
 *
 * @template T The type of result from successful execution
 *
 * @example
 * ```typescript
 * const { status, result, error } = useExecution<UploadResult>()
 * if (status === 'success') console.log(result.fileId)
 * ```
 */
export interface ExecutionHookState<T = unknown> {
  /**
   * Current execution status: idle, pending, success, or error
   */
  status: 'idle' | 'pending' | 'success' | 'error'

  /**
   * Result of successful execution, undefined if not yet complete
   */
  result: T | undefined

  /**
   * Error message if execution failed
   */
  error: string | null

  /**
   * Whether currently executing
   */
  isExecuting: boolean
}

/**
 * Standard modal/dialog hook return type.
 *
 * Used by hooks that manage modal visibility and state.
 *
 * @template T The type of data passed to the modal
 *
 * @example
 * ```typescript
 * const { isOpen, data, open, close } = useModal<ConfirmData>()
 * if (isOpen) <ConfirmModal data={data} onConfirm={close} />
 * ```
 */
export interface ModalHookState<T = unknown> {
  /**
   * Whether modal is currently visible
   */
  isOpen: boolean

  /**
   * Data passed to the modal, undefined if not open
   */
  data: T | undefined

  /**
   * Open the modal with optional data
   */
  open: (data?: T) => void

  /**
   * Close the modal
   */
  close: () => void
}

/**
 * Standard pagination state used by hooks that paginate lists.
 *
 * @example
 * ```typescript
 * const { items, page, hasNext, goToNext } = usePaginatedList(data)
 * ```
 */
export interface PaginationHookState<T> {
  /**
   * Items on current page
   */
  items: T[]

  /**
   * Current page number (0-indexed)
   */
  page: number

  /**
   * Total number of items
   */
  total: number

  /**
   * Items per page
   */
  pageSize: number

  /**
   * Whether there is a next page
   */
  hasNext: boolean

  /**
   * Whether there is a previous page
   */
  hasPrev: boolean
}

/**
 * Standard filter hook return type.
 *
 * Used by hooks that manage filtered lists and provide filter controls.
 *
 * @template T The type of items being filtered
 * @template F The type of filter configuration
 *
 * @example
 * ```typescript
 * const { items, filters, setFilters } = useFilters(data, defaultFilters)
 * ```
 */
export interface FilterHookState<T, F> {
  /**
   * Filtered items matching current filters
   */
  items: T[]

  /**
   * Current filter configuration
   */
  filters: F

  /**
   * Update filters (partial update)
   */
  setFilters: (filters: Partial<F>) => void

  /**
   * Reset filters to defaults
   */
  resetFilters: () => void

  /**
   * Total unfiltered item count
   */
  total: number
}
