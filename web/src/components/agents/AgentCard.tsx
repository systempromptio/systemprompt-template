import type { AgentCard as AgentCardType } from '@a2a-js/sdk'
import {
  AuthRequirementSection,
  ServiceStatusSection,
  SystemInstructionsSection,
  VersionProviderSection,
  CapabilitiesSection,
  McpServersSection,
  SkillsSection,
  SecuritySection,
  DocumentationSection,
} from './AgentCardSections'

interface AgentCardProps {
  agent: AgentCardType
}

export function AgentCard({ agent }: AgentCardProps) {
  return (
    <div className="space-y-lg text-sm font-body">
      <AuthRequirementSection agent={agent} />
      <ServiceStatusSection agent={agent} />
      <SystemInstructionsSection agent={agent} />
      <VersionProviderSection agent={agent} />
      <CapabilitiesSection agent={agent} />
      <McpServersSection agent={agent} />
      <SkillsSection agent={agent} />
      <SecuritySection agent={agent} />
      <DocumentationSection agent={agent} />
    </div>
  )
}
