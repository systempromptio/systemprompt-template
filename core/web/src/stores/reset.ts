import { useContextStore, CONTEXT_STATE } from './context.store'
import { useTaskStore } from './task.store'
import { useArtifactStore } from './artifact.store'
import { useUIStateStore } from './ui-state.store'
import { useToolsStore } from './tools.store'

export function resetAllStores() {
  useContextStore.setState({
    conversations: new Map(),
    currentContextId: CONTEXT_STATE.LOADING,
    isLoading: true,
    error: null,
    hasReceivedSnapshot: false,
  })

  useTaskStore.getState().reset()
  useArtifactStore.getState().reset()
  useUIStateStore.getState().reset()
  useToolsStore.getState().clearTools()
}

export function clearUserLocalStorage(userId?: string) {
  try {
    if (userId) {
      localStorage.removeItem(`systemprompt-context-${userId}`)
    }

    const authKeys = Object.keys(localStorage).filter(key =>
      key.startsWith('systemprompt-context-')
    )
    authKeys.forEach(key => localStorage.removeItem(key))

    localStorage.removeItem('systemprompt-current-context-id')
  } catch (error) {
    // Silently ignore localStorage errors
  }
}
