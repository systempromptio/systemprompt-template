import { Component } from 'react'
import type { ErrorInfo, ReactNode } from 'react'
import { AlertCircle, RefreshCw, Home, ChevronDown, ChevronUp } from 'lucide-react'

interface Props {
  children: ReactNode
  fallback?: (error: Error, errorInfo: ErrorInfo | null, retry: () => void) => ReactNode
  onError?: (error: Error, errorInfo: ErrorInfo) => void
  fallbackVariant?: 'full' | 'inline' | 'compact'
  showDetails?: boolean
  retryable?: boolean
  resetKeys?: unknown[]
}

interface State {
  hasError: boolean
  error: Error | null
  errorInfo: ErrorInfo | null
  retryCount: number
  showStackTrace: boolean
}

export class ErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props)
    this.state = {
      hasError: false,
      error: null,
      errorInfo: null,
      retryCount: 0,
      showStackTrace: false,
    }
  }

  static getDerivedStateFromError(error: Error): Partial<State> {
    return { hasError: true, error }
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    this.setState({
      error,
      errorInfo,
    })

    this.props.onError?.(error, errorInfo)

    console.error('ErrorBoundary caught an error:', error, errorInfo)
  }

  componentDidUpdate(prevProps: Props) {
    const resetKeys = this.props.resetKeys
    const prevResetKeys = prevProps.resetKeys

    if (
      this.state.hasError &&
      resetKeys !== prevResetKeys &&
      JSON.stringify(resetKeys) !== JSON.stringify(prevResetKeys)
    ) {
      this.reset()
    }
  }

  reset = () => {
    this.setState({
      hasError: false,
      error: null,
      errorInfo: null,
      retryCount: 0,
      showStackTrace: false,
    })
  }

  retry = () => {
    this.setState((prev) => ({
      hasError: false,
      error: null,
      errorInfo: null,
      retryCount: prev.retryCount + 1,
      showStackTrace: false,
    }))
  }

  toggleStackTrace = () => {
    this.setState((prev) => ({
      showStackTrace: !prev.showStackTrace,
    }))
  }

  renderDefaultFallback() {
    const { fallbackVariant = 'full', showDetails = true, retryable = true } = this.props
    const { error, errorInfo, showStackTrace } = this.state

    if (fallbackVariant === 'compact') {
      return (
        <div className="bg-error/10 border border-error rounded-lg p-4 my-4">
          <div className="flex items-start gap-3">
            <AlertCircle className="w-5 h-5 text-error flex-shrink-0 mt-0.5" />
            <div className="flex-1 min-w-0">
              <p className="text-sm font-medium text-error">Something went wrong</p>
              <p className="text-xs text-text-secondary mt-1">{error?.message}</p>
            </div>
            {retryable && (
              <button
                onClick={this.retry}
                className="px-3 py-1 text-xs bg-error text-white rounded hover:bg-error/90 transition-fast flex items-center gap-1"
                aria-label="Retry"
              >
                <RefreshCw className="w-3 h-3" />
                Retry
              </button>
            )}
          </div>
        </div>
      )
    }

    if (fallbackVariant === 'inline') {
      return (
        <div className="bg-surface border-2 border-error rounded-lg p-6 my-4">
          <div className="flex items-start gap-4">
            <AlertCircle className="w-6 h-6 text-error flex-shrink-0" />
            <div className="flex-1">
              <h3 className="text-lg font-semibold text-error mb-2">An error occurred</h3>
              <p className="text-sm text-text-secondary mb-4">
                {error?.message || 'An unexpected error occurred'}
              </p>
              <div className="flex gap-2">
                {retryable && (
                  <button
                    onClick={this.retry}
                    className="px-4 py-2 bg-primary text-white rounded-lg hover:bg-primary/90 transition-fast flex items-center gap-2"
                    aria-label="Try again"
                  >
                    <RefreshCw className="w-4 h-4" />
                    Try Again
                  </button>
                )}
                {showDetails && errorInfo && (
                  <button
                    onClick={this.toggleStackTrace}
                    className="px-4 py-2 bg-surface-hover text-text-primary rounded-lg hover:bg-border transition-fast flex items-center gap-2"
                    aria-label={showStackTrace ? 'Hide details' : 'Show details'}
                  >
                    {showStackTrace ? <ChevronUp className="w-4 h-4" /> : <ChevronDown className="w-4 h-4" />}
                    {showStackTrace ? 'Hide' : 'Show'} Details
                  </button>
                )}
              </div>

              {showStackTrace && errorInfo && (
                <div className="mt-4 p-4 bg-background rounded-lg border border-border">
                  <h4 className="text-xs font-semibold text-text-secondary mb-2">Error Details</h4>
                  <pre className="text-xs text-text-secondary font-mono overflow-auto max-h-48">
                    {error?.stack}
                  </pre>
                </div>
              )}
            </div>
          </div>
        </div>
      )
    }

    return (
      <div className="min-h-screen bg-background flex items-center justify-center p-8">
        <div className="max-w-2xl w-full">
          <div className="bg-surface border-2 border-error rounded-xl p-8 shadow-xl">
            <div className="flex items-start gap-4 mb-6">
              <div className="p-3 bg-error/10 rounded-full">
                <AlertCircle className="w-8 h-8 text-error" />
              </div>
              <div className="flex-1">
                <h1 className="text-2xl font-bold text-error mb-2">Application Error</h1>
                <p className="text-text-secondary">
                  We're sorry, but something went wrong. The error has been logged and we'll look into it.
                </p>
              </div>
            </div>

            <div className="bg-background border border-border rounded-lg p-4 mb-6">
              <h2 className="text-sm font-semibold text-text-primary mb-2">Error Message</h2>
              <p className="text-sm text-text-secondary font-mono">
                {error?.message || 'Unknown error'}
              </p>
            </div>

            {showDetails && errorInfo && (
              <div className="mb-6">
                <button
                  onClick={this.toggleStackTrace}
                  className="w-full flex items-center justify-between p-3 bg-surface-hover hover:bg-border rounded-lg transition-fast text-text-primary"
                  aria-expanded={showStackTrace}
                  aria-label="Toggle technical details"
                >
                  <span className="text-sm font-medium">Technical Details</span>
                  {showStackTrace ? <ChevronUp className="w-4 h-4" /> : <ChevronDown className="w-4 h-4" />}
                </button>

                {showStackTrace && (
                  <div className="mt-2 space-y-3">
                    <div className="bg-background border border-border rounded-lg p-4">
                      <h3 className="text-xs font-semibold text-text-secondary mb-2">Stack Trace</h3>
                      <pre className="text-xs text-text-secondary font-mono overflow-auto max-h-64">
                        {error?.stack}
                      </pre>
                    </div>

                    {errorInfo.componentStack && (
                      <div className="bg-background border border-border rounded-lg p-4">
                        <h3 className="text-xs font-semibold text-text-secondary mb-2">Component Stack</h3>
                        <pre className="text-xs text-text-secondary font-mono overflow-auto max-h-64">
                          {errorInfo.componentStack}
                        </pre>
                      </div>
                    )}
                  </div>
                )}
              </div>
            )}

            <div className="flex gap-3">
              {retryable && (
                <button
                  onClick={this.retry}
                  className="flex-1 px-6 py-3 bg-primary text-white rounded-lg hover:bg-primary/90 transition-fast flex items-center justify-center gap-2 font-medium"
                  aria-label="Try again"
                >
                  <RefreshCw className="w-5 h-5" />
                  Try Again
                </button>
              )}
              <button
                onClick={() => (window.location.href = '/')}
                className="flex-1 px-6 py-3 bg-surface-hover text-text-primary rounded-lg hover:bg-border transition-fast flex items-center justify-center gap-2 font-medium"
                aria-label="Go to home page"
              >
                <Home className="w-5 h-5" />
                Go Home
              </button>
            </div>
          </div>
        </div>
      </div>
    )
  }

  render() {
    if (this.state.hasError && this.state.error) {
      if (this.props.fallback) {
        return this.props.fallback(this.state.error, this.state.errorInfo, this.retry)
      }

      return this.renderDefaultFallback()
    }

    return this.props.children
  }
}
