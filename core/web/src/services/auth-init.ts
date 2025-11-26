/**
 * Singleton auth initialization guard
 * Prevents multiple parallel auth initializations
 */

let isInitializing = false
let initializationPromise: Promise<boolean> | null = null

export function resetAuthInitialization() {
  isInitializing = false
  initializationPromise = null
}

export function isAuthInitializing(): boolean {
  return isInitializing
}

export function getInitializationPromise(): Promise<boolean> | null {
  return initializationPromise
}

export function setAuthInitializing(promise: Promise<boolean>) {
  isInitializing = true
  initializationPromise = promise

  promise.finally(() => {
    isInitializing = false
    initializationPromise = null
  })
}
