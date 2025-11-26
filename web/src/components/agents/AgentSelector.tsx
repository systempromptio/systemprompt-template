import React, { useRef } from 'react'
import type { AgentCard as AgentCardType } from '@a2a-js/sdk'
import { useAgentStore } from '@/stores/agent.store'
import { useAuthStore } from '@/stores/auth.store'
import { useAuth } from '@/hooks/useAuth'
import { SelectableCard } from '@/components/ui/SelectableCard'
import { AuthRequiredBadge } from '@/components/ui/AuthRequiredBadge'
import { Avatar, SectionTitle } from '@/components/ui'
import { Globe, ShieldAlert } from 'lucide-react'
import { requiresOAuth, getServiceStatus, ServiceStatus, hasRequiredScopes, getRequiredScopes } from '@/lib/utils/a2a-extensions'
import { useArrowNavigation } from '@/lib/accessibility'

export function AgentSelector() {
  const { agents, selectedAgentUrl, selectAgent, loading, error } = useAgentStore()
  const userScopes = useAuthStore((state) => state.scopes)
  const itemRefs = useRef<HTMLDivElement[]>([])

  const handleSelectAgent = (agent: AgentCardType) => {
    selectAgent(agent.url, agent)
  }

  // Filter agents based on user permissions
  const authorizedAgents = agents.filter((agent) => {
    // If agent doesn't require OAuth, show it
    if (!requiresOAuth(agent)) return true

    // Otherwise, check if user has required scopes
    return hasRequiredScopes(agent, userScopes)
  })

  useArrowNavigation(itemRefs, {
    orientation: 'vertical',
    loop: true
  })

  return (
    <>
      {/* Error message */}
      {error && (
        <div className="p-md bg-error/10 border-b border-error/30" role="alert" aria-live="polite">
          <div className="text-sm text-error font-body">{error}</div>
        </div>
      )}

      {/* Agent list */}
      {authorizedAgents.length === 0 ? (
        <div className="text-center py-8 text-text-secondary">
          <Globe className="w-12 h-12 mx-auto mb-sm text-primary/30" aria-hidden="true" />
          <p className="text-sm font-body">
            {loading ? 'Discovering agents...' : agents.length > 0 ? 'No accessible agents' : 'No agents available'}
          </p>
        </div>
      ) : (
        <div className="space-y-sm p-md" role="listbox" aria-label="Available agents">
          {authorizedAgents.map((agent, index) => (
            <AgentItem
              key={agent.url}
              agent={agent}
              isSelected={selectedAgentUrl === agent.url}
              onSelect={() => handleSelectAgent(agent)}
              ref={(el) => {
                if (el) itemRefs.current[index] = el
              }}
              tabIndex={selectedAgentUrl === agent.url ? 0 : -1}
            />
          ))}
        </div>
      )}
    </>
  )
}

interface AgentItemProps {
  agent: AgentCardType
  isSelected: boolean
  onSelect: () => void
  tabIndex?: number
}

const AgentItem = React.forwardRef<HTMLDivElement, AgentItemProps>(
  function AgentItem({ agent, isSelected, onSelect, tabIndex = -1 }, ref) {
    const { isAuthenticated, requireAuth } = useAuth()
    const userScopes = useAuthStore((state) => state.scopes)

    const oauthRequired = requiresOAuth(agent)
    const serviceStatus = getServiceStatus(agent)
    const hasSufficientScopes = hasRequiredScopes(agent, userScopes)

    const isLocked = oauthRequired && (!isAuthenticated || !hasSufficientScopes)
    const lockReason = !isAuthenticated ? 'auth-required' : !hasSufficientScopes ? 'insufficient-scopes' : null

    const handleClick = () => {
      if (lockReason === 'auth-required') {
        requireAuth(agent.name, () => onSelect())
      } else if (lockReason === 'insufficient-scopes') {
        const allRequiredScopes = getRequiredScopes(agent)
        alert(`Insufficient permissions. This agent accepts: ${allRequiredScopes.join(' OR ')}. You have: ${userScopes.join(', ')}`)
      } else {
        onSelect()
      }
    }

    return (
      <>
        <div
          ref={ref}
          role="option"
          aria-selected={isSelected}
          tabIndex={tabIndex}
          onKeyDown={(e) => {
            if (e.key === 'Enter' || e.key === ' ') {
              e.preventDefault()
              handleClick()
            }
          }}
        >
          <SelectableCard
            selected={isSelected}
            disabled={isLocked}
            onClick={handleClick}
            bordered={false}
            aria-label={`${agent.name}${isLocked ? ` (${lockReason === 'auth-required' ? 'requires authentication' : 'insufficient permissions'})` : ''}`}
          >
            <div className="px-md py-sm">
              <div className="flex items-center gap-sm min-w-0">
                <div className="flex-shrink-0">
                  <Avatar
                    variant="agent"
                    agentName={agent.name}
                    agentId={agent.url}
                    size="sm"
                  />
                </div>
                {serviceStatus.status === ServiceStatus.Running && (
                  <div
                    className="w-2 h-2 rounded-full bg-success animate-pulse flex-shrink-0"
                    role="status"
                    aria-label="Agent running"
                  />
                )}
                <SectionTitle className="truncate flex-1 min-w-0">
                  {agent.name}
                </SectionTitle>
                {lockReason === 'insufficient-scopes' && (
                  <div className="flex items-center gap-xs px-sm py-xs bg-error/20 text-error rounded text-xs font-medium">
                    <ShieldAlert className="w-3 h-3" aria-hidden="true" />
                    <span>Insufficient Permissions</span>
                  </div>
                )}
                {lockReason === 'auth-required' && (
                  <AuthRequiredBadge size="sm" variant="warning" />
                )}
              </div>

              {agent.description && (
                <div className="text-sm text-text-secondary font-body line-clamp-2 leading-relaxed mt-sm">
                  {agent.description}
                </div>
              )}
            </div>
          </SelectableCard>
        </div>
      </>
    )
  }
)
