/**
 * Agent card section components extracted from AgentCard.
 *
 * @module agents/AgentCardSections
 */

import React from 'react'
import type { AgentCard as AgentCardType } from '@a2a-js/sdk'
import { Shield, Zap, Bell, History, Server, Code2 } from 'lucide-react'
import { AuthRequiredBadge } from '@/components/ui/AuthRequiredBadge'
import { Badge } from '@/components/ui/Badge'
import {
  getSystemInstructions,
  getServiceStatus,
  requiresOAuth,
  getRequiredScopes,
  formatUptime,
  getStatusColor,
  getMcpServers,
} from '@/lib/utils/a2a-extensions'

interface AgentSectionProps {
  agent: AgentCardType
}

export const AuthRequirementSection = React.memo(function AuthRequirementSection({
  agent,
}: AgentSectionProps) {
  const oauthRequired = requiresOAuth(agent)
  const requiredScopes = getRequiredScopes(agent)

  if (!oauthRequired) return null

  return (
    <div className="flex items-center gap-sm">
      <AuthRequiredBadge size="sm" variant="warning" />
      {requiredScopes.length > 0 && (
        <div className="text-xs text-text-secondary">
          Requires: <span className="font-medium text-text-primary">{requiredScopes.join(', ')}</span>
        </div>
      )}
    </div>
  )
})

export const ServiceStatusSection = React.memo(function ServiceStatusSection({
  agent,
}: AgentSectionProps) {
  const serviceStatus = getServiceStatus(agent)

  return (
    <div className="flex items-center justify-between">
      <div className="flex items-center gap-sm">
        <div className={`px-sm py-xs rounded text-xs font-medium border ${getStatusColor(serviceStatus.status)}`}>
          {serviceStatus.status}
        </div>
        {serviceStatus.port && (
          <div className="text-xs text-text-secondary">
            Port: <span className="font-mono text-text-primary">{serviceStatus.port}</span>
          </div>
        )}
      </div>
      {serviceStatus.uptimeSeconds && (
        <div className="text-xs text-text-secondary">
          Uptime: {formatUptime(serviceStatus.uptimeSeconds)}
        </div>
      )}
    </div>
  )
})

export const SystemInstructionsSection = React.memo(function SystemInstructionsSection({
  agent,
}: AgentSectionProps) {
  const systemInstructions = getSystemInstructions(agent)

  if (!systemInstructions) return null

  return (
    <div className="border-t border-primary/10 pt-md">
      <div className="text-text-secondary mb-sm text-sm font-medium flex items-center gap-sm">
        <Code2 className="w-4 h-4" />
        System Instructions:
      </div>
      <div className="bg-surface-dark/5 rounded-lg p-md">
        <pre className="text-xs text-text-primary font-mono whitespace-pre-wrap leading-relaxed">
          {systemInstructions}
        </pre>
      </div>
    </div>
  )
})

export const VersionProviderSection = React.memo(function VersionProviderSection({
  agent,
}: AgentSectionProps) {
  return (
    <div className="border-t border-primary/10 pt-md">
      <div className="grid grid-cols-2 gap-sm">
        <div>
          <span className="text-text-secondary text-sm">Version:</span>
          <span className="ml-sm font-mono text-text-primary text-sm">{agent.version}</span>
        </div>
        {agent.provider && (
          <div>
            <span className="text-text-secondary text-sm">Provider:</span>
            <a
              href={agent.provider.url}
              target="_blank"
              rel="noopener noreferrer"
              className="ml-sm text-primary hover:text-secondary transition-fast text-sm"
            >
              {agent.provider.organization}
            </a>
          </div>
        )}
      </div>
    </div>
  )
})

export const CapabilitiesSection = React.memo(function CapabilitiesSection({
  agent,
}: AgentSectionProps) {
  if (!agent.capabilities) return null

  return (
    <div className="border-t border-primary/10 pt-md">
      <div className="text-text-secondary mb-sm text-sm font-medium">Capabilities:</div>
      <div className="flex flex-wrap gap-sm">
        {agent.capabilities.streaming && <Badge icon={Zap} label="Streaming" variant="success" size="sm" />}
        {agent.capabilities.pushNotifications && <Badge icon={Bell} label="Push" variant="success" size="sm" />}
        {agent.capabilities.stateTransitionHistory && <Badge icon={History} label="History" variant="success" size="sm" />}
      </div>
    </div>
  )
})

export const McpServersSection = React.memo(function McpServersSection({
  agent,
}: AgentSectionProps) {
  const mcpServers = getMcpServers(agent)

  if (mcpServers.length === 0) return null

  return (
    <div className="border-t border-primary/10 pt-md">
      <div className="text-text-secondary mb-sm text-sm font-medium flex items-center gap-sm">
        <Server className="w-4 h-4" />
        MCP Servers ({mcpServers.length}):
      </div>
      <div className="space-y-sm">
        {mcpServers.map((server) => (
          <div key={server.name} className="bg-surface-dark/5 rounded-lg p-sm flex items-center justify-between">
            <div>
              <div className="font-heading text-sm uppercase tracking-wide text-text-primary">{server.name}</div>
              <div className="text-xs text-text-secondary">
                v{server.version} • {server.transport}
              </div>
            </div>
            {server.enabled && (
              <div className="px-sm py-xs bg-success/20 text-success rounded text-xs font-medium">
                Active
              </div>
            )}
          </div>
        ))}
      </div>
    </div>
  )
})

export const SkillsSection = React.memo(function SkillsSection({
  agent,
}: AgentSectionProps) {
  if (!agent.skills || agent.skills.length === 0) return null

  return (
    <div className="border-t border-primary/10 pt-md">
      <div className="text-text-secondary mb-sm text-sm font-medium">
        Skills ({agent.skills.length}):
      </div>
      <div className="space-y-sm">
        {agent.skills.slice(0, 3).map((skill) => (
          <div key={skill.id} className="bg-surface-dark/5 rounded-lg p-sm">
            <div className="font-heading text-sm uppercase tracking-wide text-text-primary mb-xs">{skill.name}</div>
            <div className="text-text-secondary line-clamp-2 text-sm leading-relaxed">{skill.description}</div>
            {skill.tags && skill.tags.length > 0 && (
              <div className="flex flex-wrap gap-xs mt-sm">
                {skill.tags.slice(0, 3).map((tag) => (
                  <span key={tag} className="px-sm py-xs bg-primary/20 text-primary rounded text-xs font-medium">
                    {tag}
                  </span>
                ))}
              </div>
            )}
          </div>
        ))}
        {agent.skills.length > 3 && (
          <div className="text-center text-text-secondary text-sm">+{agent.skills.length - 3} more</div>
        )}
      </div>
    </div>
  )
})

export const SecuritySection = React.memo(function SecuritySection({
  agent,
}: AgentSectionProps) {
  const oauthRequired = requiresOAuth(agent)

  if (!agent.securitySchemes || oauthRequired || Object.keys(agent.securitySchemes).length === 0) return null

  return (
    <div className="border-t border-primary/10 pt-md">
      <div className="flex items-center gap-sm text-text-secondary mb-sm text-sm font-medium">
        <Shield className="w-4 h-4" />
        Security:
      </div>
      <div className="flex flex-wrap gap-sm">
        {Object.entries(agent.securitySchemes).map(([key, scheme]) => (
          <span key={key} className="px-sm py-xs bg-warning/20 text-warning rounded text-xs font-medium">
            {scheme.type || key}
          </span>
        ))}
      </div>
    </div>
  )
})

export const DocumentationSection = React.memo(function DocumentationSection({
  agent,
}: AgentSectionProps) {
  if (!agent.documentationUrl) return null

  return (
    <div className="border-t border-primary/10 pt-md">
      <a
        href={agent.documentationUrl}
        target="_blank"
        rel="noopener noreferrer"
        className="text-primary hover:text-secondary transition-fast text-sm font-medium"
      >
        Documentation →
      </a>
    </div>
  )
})
