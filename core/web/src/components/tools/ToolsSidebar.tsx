import { useMcpRegistry } from '@/hooks/useMcpRegistry'
import { useToolParameters } from '@/hooks/useToolParameters'
import { useToolsStore, type McpTool } from '@/stores/tools.store'
import { useAuth } from '@/hooks/useAuth'
import { ToolParameterModal } from './ToolParameterModal'
import { Card } from '@/components/ui/Card'
import { SectionTitle } from '@/components/ui/SectionTitle'
import { AuthRequiredBadge } from '@/components/ui/AuthRequiredBadge'
import { Database, Bot, ChevronDown, ChevronRight } from 'lucide-react'
import { useState } from 'react'

function getServerIcon(serverName: string) {
  const nameLower = serverName.toLowerCase()
  if (nameLower.includes('agent') || nameLower.includes('bot')) {
    return Bot
  }
  return Database
}

export function ToolsSidebar() {
  const { tools, servers, loading, error } = useToolsStore()
  const { executeTool, submitParameters, closeModal, showModal, selectedTool } = useToolParameters()
  const { isAuthenticated, requireAuth } = useAuth()
  const [expandedServers, setExpandedServers] = useState<Set<string>>(new Set())

  useMcpRegistry()

  const toggleServer = (serverName: string) => {
    setExpandedServers((prev) => {
      const next = new Set(prev)
      if (next.has(serverName)) {
        next.delete(serverName)
      } else {
        next.add(serverName)
      }
      return next
    })
  }

  const handleToolClick = async (tool: typeof tools[0]) => {
    try {
      await executeTool(tool)
    } catch (error) {
      // Error handling done by executeTool
    }
  }

  const toolsByServer = tools.reduce((acc, tool) => {
    if (!acc[tool.serverName]) {
      acc[tool.serverName] = []
    }
    acc[tool.serverName].push(tool)
    return acc
  }, {} as Record<string, McpTool[]>)

  return (
    <>
      <div className="w-full h-full">
        {loading && (
          <div className="text-center text-text-secondary py-8">
            <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary mx-auto mb-sm"></div>
            <p className="text-sm font-body">Loading tools...</p>
          </div>
        )}

        {error && (
          <div className="p-md bg-error/10 border-b border-error/30">
            <p className="text-sm text-error font-body">{error}</p>
          </div>
        )}

        {!loading && !error && (
          <div className="space-y-sm p-md">
            {servers.map((server) => {
              const serverTools = toolsByServer[server.name] || []
              const isExpanded = expandedServers.has(server.name)
              const hasTools = serverTools.length > 0
              const isActive = server.status === 'enabled' && hasTools
              const requiresOAuth = server.oauth_required || false
              const isLocked = requiresOAuth && !isAuthenticated

              return (
                <Card
                  key={server.name}
                  variant="accent"
                  padding="none"
                  elevation="sm"
                  className={isLocked ? 'opacity-50 grayscale' : !isExpanded ? 'transition-all duration-fast hover:border-primary/60 hover:scale-105 cursor-pointer' : 'transition-all duration-fast cursor-pointer'}
                >
                  <button
                    onClick={() => {
                      if (isLocked) {
                        requireAuth()
                      } else {
                        toggleServer(server.name)
                      }
                    }}
                    className="w-full flex items-center justify-between p-md hover:bg-primary/5 transition-fast cursor-pointer min-h-[64px]"
                  >
                    <div className="flex items-center gap-sm flex-1 min-w-0">
                      {(() => {
                        const IconComponent = getServerIcon(server.name)
                        return <IconComponent className="w-4 h-4 text-primary flex-shrink-0" />
                      })()}
                      <div className="flex-1 min-w-0 text-left">
                        <div className="flex items-center gap-sm mb-xs">
                          <SectionTitle className="truncate">{server.name}</SectionTitle>
                          {requiresOAuth && (
                            <AuthRequiredBadge
                              size="sm"
                              variant="warning"
                              label={server.oauth_scopes?.join(', ') || 'OAuth'}
                            />
                          )}
                        </div>
                        <div className="flex items-center gap-sm text-xs text-text-secondary font-body">
                          <span>{serverTools.length} tool{serverTools.length !== 1 ? 's' : ''}</span>
                          <span
                            className={`inline-block w-2 h-2 rounded-full ${
                              isActive ? 'bg-success' : 'bg-text-disabled'
                            }`}
                          />
                        </div>
                      </div>
                    </div>
                    {hasTools && (
                      isExpanded ? (
                        <ChevronDown className="w-4 h-4 text-primary flex-shrink-0" />
                      ) : (
                        <ChevronRight className="w-4 h-4 text-primary flex-shrink-0" />
                      )
                    )}
                  </button>

                  <div
                    className="overflow-hidden transition-all duration-300"
                    style={{
                      maxHeight: isExpanded && hasTools ? '1000px' : '0',
                    }}
                  >
                    <div className="bg-surface-dark/5 border-t border-primary/10">
                      {serverTools.map((tool, index) => (
                        <button
                          key={`${tool.serverName}-${tool.name}`}
                          onClick={() => handleToolClick(tool)}
                          className={`w-full text-left p-md cursor-pointer ${
                            index !== serverTools.length - 1 ? 'border-b border-primary/10' : ''
                          }`}
                        >
                          <div className="font-medium text-sm text-text-primary mb-xs font-body">
                            {tool.name}
                            <span className="ml-sm text-xs px-xs py-0.5 bg-success/20 text-success rounded">
                              Live
                            </span>
                          </div>
                          {tool.description && (
                            <p className="text-xs text-text-secondary mb-sm font-body">{tool.description}</p>
                          )}
                          <div className="text-xs text-text-disabled font-mono">
                            {Object.keys(tool.inputSchema).length} parameter
                            {Object.keys(tool.inputSchema).length !== 1 ? 's' : ''}
                          </div>
                        </button>
                      ))}
                    </div>
                  </div>
                </Card>
              )
            })}
          </div>
        )}
      </div>

      {/* Tool Parameter Modal */}
      <ToolParameterModal
        isOpen={showModal}
        tool={selectedTool}
        onClose={closeModal}
        onSubmit={submitParameters}
      />
    </>
  )
}
