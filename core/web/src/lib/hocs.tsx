import { Component, type ComponentType, type ReactNode } from 'react'
import { ErrorBoundary } from '@/components/ErrorBoundary'
import { LoadingBoundary, LoadingSpinner } from '@/components/LoadingBoundary'
import type { ErrorInfo } from 'react'

export interface WithErrorBoundaryOptions {
  fallback?: (error: Error, errorInfo: ErrorInfo | null, retry: () => void) => ReactNode
  onError?: (error: Error, errorInfo: ErrorInfo) => void
  fallbackVariant?: 'full' | 'inline' | 'compact'
  showDetails?: boolean
  retryable?: boolean
}

export function withErrorBoundary<P extends object>(
  WrappedComponent: ComponentType<P>,
  options: WithErrorBoundaryOptions = {}
): ComponentType<P> {
  return class WithErrorBoundary extends Component<P> {
    static displayName = `withErrorBoundary(${WrappedComponent.displayName || WrappedComponent.name || 'Component'})`

    render() {
      return (
        <ErrorBoundary
          fallback={options.fallback}
          onError={options.onError}
          fallbackVariant={options.fallbackVariant || 'inline'}
          showDetails={options.showDetails ?? true}
          retryable={options.retryable ?? true}
        >
          <WrappedComponent {...this.props} />
        </ErrorBoundary>
      )
    }
  }
}

export interface WithLoadingOptions {
  fallback?: ReactNode
  loadingVariant?: 'full' | 'inline' | 'spinner' | 'skeleton'
  loadingText?: string
  minHeight?: string
}

export function withLoading<P extends { isLoading?: boolean }>(
  WrappedComponent: ComponentType<P>,
  options: WithLoadingOptions = {}
): ComponentType<P> {
  return class WithLoading extends Component<P> {
    static displayName = `withLoading(${WrappedComponent.displayName || WrappedComponent.name || 'Component'})`

    render() {
      const { isLoading, ...restProps } = this.props as P & { isLoading?: boolean }

      if (isLoading) {
        if (options.fallback) {
          return <>{options.fallback}</>
        }

        if (options.loadingVariant === 'spinner') {
          return <LoadingSpinner size="md" text={options.loadingText} />
        }

        return (
          <LoadingBoundary
            loadingVariant={options.loadingVariant || 'inline'}
            loadingText={options.loadingText}
            minHeight={options.minHeight}
          >
            {null}
          </LoadingBoundary>
        )
      }

      return <WrappedComponent {...(restProps as P)} />
    }
  }
}

export interface WithAuthOptions {
  fallback?: ReactNode
  redirectTo?: string
  checkAuth?: () => boolean | Promise<boolean>
  loadingComponent?: ReactNode
}

export function withAuth<P extends object>(
  WrappedComponent: ComponentType<P>,
  options: WithAuthOptions = {}
): ComponentType<P> {
  return class WithAuth extends Component<P, { isAuthenticated: boolean | null }> {
    static displayName = `withAuth(${WrappedComponent.displayName || WrappedComponent.name || 'Component'})`

    constructor(props: P) {
      super(props)
      this.state = {
        isAuthenticated: null,
      }
    }

    async componentDidMount() {
      await this.checkAuthentication()
    }

    async checkAuthentication() {
      if (options.checkAuth) {
        try {
          const isAuth = await Promise.resolve(options.checkAuth())
          this.setState({ isAuthenticated: isAuth })

          if (!isAuth && options.redirectTo) {
            window.location.href = options.redirectTo
          }
        } catch {
          this.setState({ isAuthenticated: false })
        }
      } else {
        const token = localStorage.getItem('auth_token')
        const isAuth = !!token

        this.setState({ isAuthenticated: isAuth })

        if (!isAuth && options.redirectTo) {
          window.location.href = options.redirectTo
        }
      }
    }

    render() {
      const { isAuthenticated } = this.state

      if (isAuthenticated === null) {
        return (
          options.loadingComponent || (
            <LoadingSpinner size="lg" text="Checking authentication..." />
          )
        )
      }

      if (!isAuthenticated) {
        if (options.fallback) {
          return <>{options.fallback}</>
        }

        return (
          <div className="min-h-screen bg-background flex items-center justify-center p-8">
            <div className="bg-surface border border-border rounded-lg p-8 max-w-md w-full text-center">
              <h2 className="text-xl font-bold text-text-primary mb-4">Authentication Required</h2>
              <p className="text-text-secondary mb-6">
                You need to be logged in to access this page.
              </p>
              <button
                onClick={() => (window.location.href = options.redirectTo || '/login')}
                className="px-6 py-3 bg-primary text-white rounded-lg hover:bg-primary/90 transition-fast font-medium"
              >
                Go to Login
              </button>
            </div>
          </div>
        )
      }

      return <WrappedComponent {...this.props} />
    }
  }
}

export interface WithErrorAndLoadingOptions extends Omit<WithErrorBoundaryOptions, 'fallback'>, Omit<WithLoadingOptions, 'fallback'> {
  errorFallback?: (error: Error, errorInfo: ErrorInfo | null, retry: () => void) => ReactNode
  loadingFallback?: ReactNode
}

export function withErrorAndLoading<P extends { isLoading?: boolean }>(
  WrappedComponent: ComponentType<P>,
  options: WithErrorAndLoadingOptions = {}
): ComponentType<P> {
  const ComponentWithLoading = withLoading(WrappedComponent, {
    fallback: options.loadingFallback,
    loadingVariant: options.loadingVariant,
    loadingText: options.loadingText,
    minHeight: options.minHeight,
  })

  return withErrorBoundary(ComponentWithLoading, {
    fallback: options.errorFallback,
    onError: options.onError,
    fallbackVariant: options.fallbackVariant,
    showDetails: options.showDetails,
    retryable: options.retryable,
  })
}

export interface WithRetryOptions {
  maxRetries?: number
  retryDelay?: number
  onRetry?: (attempt: number) => void
  onMaxRetriesReached?: () => void
}

export function withRetry<P extends { onRetry?: () => void; retryCount?: number }>(
  WrappedComponent: ComponentType<P>,
  options: WithRetryOptions = {}
): ComponentType<Omit<P, 'onRetry' | 'retryCount'>> {
  return class WithRetry extends Component<Omit<P, 'onRetry' | 'retryCount'>, { retryCount: number }> {
    static displayName = `withRetry(${WrappedComponent.displayName || WrappedComponent.name || 'Component'})`

    private retryTimeout: number | null = null

    constructor(props: Omit<P, 'onRetry' | 'retryCount'>) {
      super(props)
      this.state = {
        retryCount: 0,
      }
    }

    componentWillUnmount() {
      if (this.retryTimeout !== null) {
        clearTimeout(this.retryTimeout)
      }
    }

    handleRetry = () => {
      const { maxRetries = 3, retryDelay = 1000, onRetry, onMaxRetriesReached } = options
      const nextRetryCount = this.state.retryCount + 1

      if (nextRetryCount > maxRetries) {
        onMaxRetriesReached?.()
        return
      }

      this.setState({ retryCount: nextRetryCount })
      onRetry?.(nextRetryCount)

      if (retryDelay > 0) {
        this.retryTimeout = window.setTimeout(() => {
          this.forceUpdate()
        }, retryDelay)
      }
    }

    render() {
      return (
        <WrappedComponent
          {...(this.props as P)}
          onRetry={this.handleRetry}
          retryCount={this.state.retryCount}
        />
      )
    }
  }
}

export interface WithPerformanceOptions {
  logName?: string
  logThreshold?: number
}

export function withPerformance<P extends object>(
  WrappedComponent: ComponentType<P>,
  options: WithPerformanceOptions = {}
): ComponentType<P> {
  return class WithPerformance extends Component<P> {
    static displayName = `withPerformance(${WrappedComponent.displayName || WrappedComponent.name || 'Component'})`

    private mountTime: number | null = null

    componentDidMount() {
      this.mountTime = performance.now()
      const renderTime = this.mountTime - (window.performance?.timing?.navigationStart || 0)

      const componentName =
        options.logName ||
        WrappedComponent.displayName ||
        WrappedComponent.name ||
        'Component'

      if (options.logThreshold && renderTime > options.logThreshold) {
        console.warn(`${componentName} took ${renderTime.toFixed(2)}ms to mount`)
      }
    }

    componentWillUnmount() {
      if (this.mountTime !== null) {
        const unmountTime = performance.now()
        const lifeTime = unmountTime - this.mountTime

        if (options.logThreshold && lifeTime < options.logThreshold) {
          const componentName =
            options.logName ||
            WrappedComponent.displayName ||
            WrappedComponent.name ||
            'Component'

          console.warn(`${componentName} was unmounted quickly (${lifeTime.toFixed(2)}ms)`)
        }
      }
    }

    render() {
      return <WrappedComponent {...this.props} />
    }
  }
}

type ComposeHOC = <P extends object>(component: ComponentType<P>) => ComponentType<P>

export function compose(...hocs: ComposeHOC[]): ComposeHOC {
  return (component) => hocs.reduceRight((acc, hoc) => hoc(acc), component)
}

export function withAll<P extends object>(
  WrappedComponent: ComponentType<P>,
  options: {
    error?: WithErrorBoundaryOptions
    loading?: WithLoadingOptions
    auth?: WithAuthOptions
    retry?: WithRetryOptions
    performance?: WithPerformanceOptions
  } = {}
): ComponentType<P> {
  let Component = WrappedComponent

  if (options.performance) {
    Component = withPerformance(Component, options.performance)
  }

  if (options.retry) {
    Component = withRetry(Component as ComponentType<P & { onRetry?: () => void; retryCount?: number }>, options.retry) as ComponentType<P>
  }

  if (options.loading) {
    Component = withLoading(Component as ComponentType<P & { isLoading?: boolean }>, options.loading) as ComponentType<P>
  }

  if (options.error) {
    Component = withErrorBoundary(Component, options.error)
  }

  if (options.auth) {
    Component = withAuth(Component, options.auth)
  }

  return Component
}
