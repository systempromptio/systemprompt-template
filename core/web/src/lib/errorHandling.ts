interface ErrorInfo {
  message: string
  stack?: string
  timestamp: Date
  source: 'uncaught' | 'unhandledRejection' | 'react'
}

type ErrorCallback = (error: ErrorInfo) => void

class GlobalErrorHandler {
  private callbacks: Set<ErrorCallback> = new Set()
  private initialized = false

  subscribe(callback: ErrorCallback): () => void {
    this.callbacks.add(callback)
    return () => this.callbacks.delete(callback)
  }

  private notify(error: ErrorInfo) {
    this.callbacks.forEach((callback) => {
      try {
        callback(error)
      } catch (err) {
      }
    })
  }

  initialize() {
    if (this.initialized) {
      return
    }

    window.addEventListener('error', (event: ErrorEvent) => {

      this.notify({
        message: event.error?.message || event.message || 'Unknown error',
        stack: event.error?.stack,
        timestamp: new Date(),
        source: 'uncaught',
      })

      event.preventDefault()
    })

    window.addEventListener('unhandledrejection', (event: PromiseRejectionEvent) => {

      const error = event.reason instanceof Error
        ? event.reason
        : new Error(String(event.reason))

      this.notify({
        message: error.message || 'Unhandled promise rejection',
        stack: error.stack,
        timestamp: new Date(),
        source: 'unhandledRejection',
      })

      event.preventDefault()
    })

    this.initialized = true
  }

  cleanup() {
    if (!this.initialized) return

    this.callbacks.clear()
    this.initialized = false
  }
}

export const globalErrorHandler = new GlobalErrorHandler()
export type { ErrorInfo }
