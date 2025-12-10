/**
 * Root application component.
 * Orchestrates initialization of all application subsystems including authentication,
 * agent discovery, context streaming, and artifact management.
 *
 * Initialization flow:
 * 1. Anonymous auth via useAuthInitialization hook
 * 2. Agent discovery in parallel with auth
 * 3. Artifact fetching after auth succeeds
 * 4. Context initialization and SSE stream setup
 *
 * UI state management:
 * - Mobile menu toggle for responsive layout
 * - View switching between conversation and other modes
 * - Auth modal for credential management
 * - Artifact viewer modal
 *
 * Component composition:
 * - Header with navigation and user info
 * - Left sidebar with agents and tools
 * - Main content area with ViewRouter
 * - Modals for auth and artifact viewing
 *
 * @returns {JSX.Element} Root application layout with all subsystems initialized
 */

import { useEffect, useState, useRef, lazy, Suspense } from 'react'
import { ViewRouter } from '@/components/views/ViewRouter'
import { UserInfoWidget } from '@/components/auth/UserInfoWidget'
import { AppLayout, Header, HeaderMobile, LeftSidebar } from '@/components/layout'

const AgentSelector = lazy(() => import('@/components/agents/AgentSelector').then(m => ({ default: m.AgentSelector })))
const ToolsSidebar = lazy(() => import('@/components/tools/ToolsSidebar').then(m => ({ default: m.ToolsSidebar })))
const AuthModal = lazy(() => import('@/components/auth/AuthModal').then(m => ({ default: m.AuthModal })))
const ArtifactModal = lazy(() => import('@/components/artifacts/ArtifactModal').then(m => ({ default: m.ArtifactModal })))
const SkillViewer = lazy(() => import('@/components/skills/SkillViewer').then(m => ({ default: m.SkillViewer })))
import { useAgentDiscovery } from '@/hooks/useAgentDiscovery'
import { useAuth } from '@/hooks/useAuth'
import { useAuthInitialization } from '@/hooks/useAuthInitialization'
import { useContextStream } from '@/hooks/useContextStream'
import { useTokenExpiryMonitor } from '@/hooks/useTokenExpiryMonitor'
import { useContextInit } from '@/hooks/useContextInit'
import { useContextStore, CONTEXT_STATE } from '@/stores/context.store'
import { useAuthStore } from '@/stores/auth.store'
import { useAgentStore } from '@/stores/agent.store'
import { useArtifactStore } from '@/stores/artifact.store'
import { useViewStore } from '@/stores/view.store'
import { useSettingsStore } from '@/stores/settings.store'
import { ConversationToggle } from '@/components/conversations/ConversationToggle'
import { getApiUrl } from '@/utils/env'
import { theme } from '@/theme.config'

function App() {
  const agentDiscovery = useAgentDiscovery()
  useTokenExpiryMonitor()
  useContextInit()

  const { isAuthenticated, showAuthModal, handleAuthSuccess, handleAuthClose, authAgentName } = useAuth()
  const { isTokenValid, accessToken } = useAuthStore()
  const { activeView, setActiveView } = useViewStore()
  const { leftSidebarVisible } = useSettingsStore()
  const { initializeAnonymousAuth } = useAuthInitialization()
  const [isInitialized, setIsInitialized] = useState(false)
  const [authInitError, setAuthInitError] = useState<string | null>(null)
  const [mobileMenuOpen, setMobileMenuOpen] = useState(false)
  const hasAttemptedArtifactLoad = useRef(false)

  useContextStream()

  /**
   * Retry pending agent selection after agents load.
   * This handles the race condition where SSE current_agent events arrive
   * before agents are loaded from the registry.
   */
  const agentCount = useAgentStore((state) => state.agents.length)
  const selectedAgent = useAgentStore((state) => state.selectedAgent)
  useEffect(() => {
    const agents = useAgentStore.getState().agents
    const contextStore = useContextStore.getState()
    const currentContextId = contextStore.currentContextId
    const assignedAgentName = currentContextId !== 'LOADING' && currentContextId !== 'NONE'
      ? contextStore.contextAgents.get(currentContextId)
      : null

    if (agents.length > 0 && assignedAgentName && !selectedAgent) {
      const matchingAgent = agents.find((agent) =>
        agent.name.toLowerCase() === assignedAgentName.toLowerCase()
      )
      if (matchingAgent) {
        useAgentStore.getState().selectAgent(matchingAgent.url, matchingAgent)
      } else {
        const errorMsg = `Agent "${assignedAgentName}" not found. Available: ${agents.map(a => a.name).join(', ')}`
        useAgentStore.getState().setSelectionError(errorMsg)
      }
    }
  }, [agentCount, selectedAgent])

  /**
   * Initialize authentication first, then agent discovery.
   * Runs only once on component mount to set up foundational application state.
   * Auth must complete before agent discovery to ensure JWT token is available.
   */
  useEffect(() => {
    (async () => {
      try {
        await initializeAnonymousAuth()
        await agentDiscovery.refresh()
      } catch {
        setAuthInitError('Failed to initialize application. Please refresh the page.')
      }
    })()
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  /**
   * Mark application as initialized once authentication is valid.
   * Triggers UI rendering transition from loading to main interface.
   */
  useEffect(() => {
    if (accessToken && isTokenValid()) {
      setIsInitialized(true)
    }
  }, [accessToken, isTokenValid])

  /**
   * Fetch all artifacts after auth is established.
   * Deferred to allow critical UI rendering to complete first.
   * Uses 100ms timeout to deprioritize artifact loading.
   */
  useEffect(() => {
    if (hasAttemptedArtifactLoad.current) return

    const authState = useAuthStore.getState()
    if (authState.accessToken && authState.isTokenValid()) {
      hasAttemptedArtifactLoad.current = true
      const timeoutId = setTimeout(() => {
        const authHeader = authState.getAuthHeader()
        if (authHeader) {
          useArtifactStore.getState().fetchAllArtifacts(authHeader)
        }
      }, 100)
      return () => clearTimeout(timeoutId)
    }
  }, [accessToken, isTokenValid])

  /**
   * Reset context state when authentication status changes.
   * Clears conversations when user becomes unauthenticated.
   */
  useEffect(() => {
    if (!isAuthenticated) {
      useContextStore.setState({ conversations: new Map(), currentContextId: CONTEXT_STATE.LOADING })
    }
  }, [isAuthenticated])

  /**
   * Render error state when authentication initialization fails.
   * Provides user feedback and recovery option (page refresh).
   */
  if (authInitError) {
    return (
      <div className="flex h-screen items-center justify-center bg-background">
        <div className="text-center space-y-4 max-w-md px-6">
          <div className="flex items-center justify-center">
            <svg
              className="h-12 w-12 text-error"
              xmlns="http://www.w3.org/2000/svg"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
              />
            </svg>
          </div>
          <div>
            <h2 className="text-xl font-semibold text-text-primary">Authentication Failed</h2>
            <p className="text-sm text-text-secondary mt-2">{authInitError}</p>
          </div>
          <button
            onClick={() => window.location.reload()}
            className="px-4 py-2 bg-primary text-white rounded-lg hover:bg-primary/90 transition-fast"
          >
            Refresh Page
          </button>
        </div>
      </div>
    )
  }

  /**
   * Render loading state while application initializes.
   * Displays spinner and status message until auth and core services are ready.
   */
  if (!isInitialized) {
    return (
      <div className="flex h-screen items-center justify-center bg-background">
        <div className="text-center space-y-4">
          <div className="flex items-center justify-center">
            <svg
              className="animate-spin h-12 w-12 text-primary"
              xmlns="http://www.w3.org/2000/svg"
              fill="none"
              viewBox="0 0 24 24"
            >
              <circle
                className="opacity-25"
                cx="12"
                cy="12"
                r="10"
                stroke="currentColor"
                strokeWidth="4"
              />
              <path
                className="opacity-75"
                fill="currentColor"
                d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
              />
            </svg>
          </div>
          <div>
            <h2 className="text-xl font-semibold text-text-primary">Initializing {theme.branding.name}</h2>
            <p className="text-sm text-text-secondary mt-2">Setting up your session...</p>
          </div>
        </div>
      </div>
    )
  }

  return (
    <>
      <AppLayout
        activeView={activeView}
        onViewChange={setActiveView}
        showLeftSidebar={leftSidebarVisible}
        mobileMenuOpen={mobileMenuOpen}
        onMobileMenuClose={() => setMobileMenuOpen(false)}
        header={
          <>
            <HeaderMobile
              onMenuClick={() => setMobileMenuOpen(true)}
              centerContent={
                <ConversationToggle
                  isSelected={activeView === 'conversation'}
                  onViewChange={() => setActiveView('conversation')}
                />
              }
            />
            <Header
              centerContent={
                <ConversationToggle
                  isSelected={activeView === 'conversation'}
                  onViewChange={() => setActiveView('conversation')}
                />
              }
              rightContent={<UserInfoWidget />}
            />
          </>
        }
        leftSidebar={
          <LeftSidebar
            agentsContent={
              <Suspense fallback={<div className="text-xs text-text-secondary p-md">Loading agents...</div>}>
                <AgentSelector />
              </Suspense>
            }
            toolsContent={
              <Suspense fallback={<div className="text-xs text-text-secondary p-md">Loading tools...</div>}>
                <ToolsSidebar />
              </Suspense>
            }
            footer={
              <div className="space-y-md">
                {/* Content Section */}
                <div>
                  <div className="text-xs font-medium font-heading text-text-secondary uppercase tracking-wide mb-sm">
                    Content
                  </div>
                  <div className="flex flex-col gap-xs text-xs font-body">
                    <a
                      href="/blog"
                      className="text-text-secondary hover:text-primary transition-fast"
                    >
                      Blog
                    </a>
                  </div>
                </div>

                {/* Registries Section */}
                <div>
                  <div className="text-xs font-medium font-heading text-text-secondary uppercase tracking-wide mb-sm">
                    Registries
                  </div>
                  <div className="flex flex-col gap-xs text-xs font-body">
                    <a
                      href={getApiUrl('/api/v1/agents/registry')}
                      target="_blank"
                      rel="noopener noreferrer"
                      className="text-text-secondary hover:text-primary transition-fast"
                    >
                      Agent Registry ↗
                    </a>
                    <a
                      href={getApiUrl('/api/v1/mcp/registry')}
                      target="_blank"
                      rel="noopener noreferrer"
                      className="text-text-secondary hover:text-primary transition-fast"
                    >
                      MCP Registry ↗
                    </a>
                    <a
                      href="/sitemap.xml"
                      target="_blank"
                      rel="noopener noreferrer"
                      className="text-text-secondary hover:text-primary transition-fast"
                    >
                      Sitemap ↗
                    </a>
                  </div>
                </div>

                {/* Divider */}
                <div className="border-t border-primary/10" />

                {/* Legal Links & Copyright */}
                <div className="text-xs font-body text-text-secondary/60">
                  <a href="/legal" className="hover:text-primary transition-fast">Legal</a>
                  {' · '}
                  <a href="/privacy" className="hover:text-primary transition-fast">Privacy</a>
                  {' · '}
                  <a href="/terms" className="hover:text-primary transition-fast">Terms</a>
                  {' · '}
                  © {theme.metadata.copyright.year} {theme.metadata.copyright.holder}
                </div>
              </div>
            }
          />
        }
      >
        <ViewRouter />
      </AppLayout>

      {showAuthModal && (
        <Suspense fallback={null}>
          <AuthModal
            isOpen={showAuthModal}
            onClose={handleAuthClose}
            onSuccess={handleAuthSuccess}
            agentName={authAgentName}
          />
        </Suspense>
      )}

      <Suspense fallback={null}>
        <ArtifactModal />
      </Suspense>

      <Suspense fallback={null}>
        <SkillViewer />
      </Suspense>
    </>
  )
}

export default App
