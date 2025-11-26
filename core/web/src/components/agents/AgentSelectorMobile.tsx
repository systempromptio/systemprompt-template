import type { AgentCard as AgentCardType } from '@a2a-js/sdk'
import { useAgentStore } from '@/stores/agent.store'
import { useAuthStore } from '@/stores/auth.store'
import { useAuth } from '@/hooks/useAuth'
import { Globe, Check } from 'lucide-react'
import { requiresOAuth, getServiceStatus, ServiceStatus, hasRequiredScopes } from '@/lib/utils/a2a-extensions'
import { cn } from '@/lib/utils/cn'
import { Avatar } from '@/components/ui/Avatar'

interface AgentSelectorMobileProps {
  onAgentSelect?: () => void
}

export function AgentSelectorMobile({ onAgentSelect }: AgentSelectorMobileProps) {
  const { agents, selectedAgentUrl, selectAgent, loading, error } = useAgentStore()
  const userScopes = useAuthStore((state) => state.scopes)

  const handleSelectAgent = (agent: AgentCardType) => {
    selectAgent(agent.url, agent)
    onAgentSelect?.()
  }

  // Filter agents based on user permissions
  const authorizedAgents = agents.filter((agent) => {
    // If agent doesn't require OAuth, show it
    if (!requiresOAuth(agent)) return true

    // Otherwise, check if user has required scopes
    return hasRequiredScopes(agent, userScopes)
  })

  return (
    <div className="flex flex-col">
      {error && (
        <div className="p-md bg-error/10 border-b border-error/30">
          <div className="text-sm text-error font-body">{error}</div>
        </div>
      )}

      {authorizedAgents.length === 0 ? (
        <div className="text-center py-8 text-text-secondary">
          <Globe className="w-12 h-12 mx-auto mb-sm text-primary/30" />
          <p className="text-sm font-body">
            {loading ? 'Discovering agents...' : agents.length > 0 ? 'No accessible agents' : 'No agents available'}
          </p>
        </div>
      ) : (
        <div className="divide-y divide-primary/10">
          {authorizedAgents.map((agent) => (
            <CompactAgentItem
              key={agent.url}
              agent={agent}
              isSelected={selectedAgentUrl === agent.url}
              onSelect={() => handleSelectAgent(agent)}
            />
          ))}
        </div>
      )}
    </div>
  )
}

interface CompactAgentItemProps {
  agent: AgentCardType
  isSelected: boolean
  onSelect: () => void
}

function CompactAgentItem({ agent, isSelected, onSelect }: CompactAgentItemProps) {
  const { isAuthenticated, requireAuth } = useAuth()

  const oauthRequired = requiresOAuth(agent)
  const serviceStatus = getServiceStatus(agent)
  const isLocked = oauthRequired && !isAuthenticated

  const icon = (
    <Avatar
      variant="agent"
      agentName={agent.name}
      agentId={agent.url}
      size="sm"
    />
  )

  const handleClick = () => {
    if (isLocked) {
      requireAuth(agent.name, () => onSelect())
    } else {
      onSelect()
    }
  }

  return (
    <button
      onClick={handleClick}
      className={cn(
        'w-full flex items-center gap-sm p-md min-h-[56px]',
        'transition-all duration-fast',
        'active:scale-[0.98]',
        isSelected && 'bg-success/5 border-l-4 border-success',
        isLocked && 'opacity-40 grayscale',
        !isLocked && 'hover:bg-primary/5'
      )}
    >
      <div className="flex-shrink-0">{icon}</div>

      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-xs justify-start">
          {serviceStatus.status === ServiceStatus.Running && (
            <div className="w-2 h-2 rounded-full bg-success animate-pulse flex-shrink-0" />
          )}
          <div className="font-heading text-sm uppercase tracking-wide text-text-primary truncate">
            {agent.name}
          </div>
        </div>
        {serviceStatus.status !== ServiceStatus.Unknown && serviceStatus.status !== ServiceStatus.Running && (
          <div className="text-xs text-text-secondary truncate">
            {serviceStatus.status}
          </div>
        )}
      </div>

      <div className="flex-shrink-0 flex items-center gap-xs">
        {isSelected && (
          <div className="w-6 h-6 bg-success/90 rounded-full flex items-center justify-center shadow-sm">
            <Check className="w-4 h-4 text-white" />
          </div>
        )}
      </div>
    </button>
  )
}
