import { useState } from 'react'
import { ChevronRight, ChevronDown, Circle, CheckCircle, AlertTriangle, XCircle, HelpCircle } from 'lucide-react'
import type { Artifact, TreeHints } from '@/types/artifact'
import type { TreeNode } from '@/lib/mcp/types'
import { extractTreeData, unwrapExtraction } from '@/lib/artifacts'

interface TreeRendererProps {
  artifact: Artifact
  hints: TreeHints
}

export function TreeRenderer({ artifact, hints }: TreeRendererProps) {
  const treeDataResult = extractTreeData(artifact)
  const treeData = unwrapExtraction(treeDataResult)

  if (!treeData) {
    return <div className="text-secondary text-center py-8">Invalid tree data</div>
  }

  const defaultExpandedLevels = hints.default_expanded_levels || 2

  return (
    <div className="space-y-2">
      <TreeNodeComponent
        node={treeData}
        level={0}
        defaultExpandedLevels={defaultExpandedLevels}
        showIcons={hints.show_icons !== false}
        iconMap={hints.icon_map}
      />
    </div>
  )
}

interface TreeNodeComponentProps {
  node: TreeNode
  level: number
  defaultExpandedLevels: number
  showIcons: boolean
  iconMap?: Record<string, string>
}

function TreeNodeComponent({
  node,
  level,
  defaultExpandedLevels,
  showIcons,
  iconMap,
}: TreeNodeComponentProps) {
  const [isExpanded, setIsExpanded] = useState(level < defaultExpandedLevels)
  const hasChildren = node.children && node.children.length > 0

  const getIcon = () => {
    if (!showIcons || !node.status) return null

    const status = node.status.toLowerCase()

    switch (status) {
      case 'healthy':
      case 'success':
        return <CheckCircle className="w-4 h-4 text-success" />
      case 'warning':
        return <AlertTriangle className="w-4 h-4 text-warning" />
      case 'error':
      case 'critical':
        return <XCircle className="w-4 h-4 text-error" />
      case 'unknown':
        return <HelpCircle className="w-4 h-4 text-disabled" />
      default:
        return <Circle className="w-4 h-4 text-disabled" />
    }
  }

  return (
    <div>
      <div
        className={`flex items-center gap-2 py-1 px-2 rounded hover:bg-surface-variant ${
          hasChildren ? 'cursor-pointer' : ''
        }`}
        style={{ paddingLeft: `${level * 1.5 + 0.5}rem` }}
        onClick={() => hasChildren && setIsExpanded(!isExpanded)}
      >
        {hasChildren ? (
          isExpanded ? (
            <ChevronDown className="w-4 h-4 text-disabled flex-shrink-0" />
          ) : (
            <ChevronRight className="w-4 h-4 text-disabled flex-shrink-0" />
          )
        ) : (
          <span className="w-4 h-4 flex-shrink-0" />
        )}

        {getIcon()}

        <span className="font-medium text-primary">{node.name}</span>

        {node.status && (
          <span className={`text-xs px-2 py-0.5 rounded ${getStatusColor(node.status)}`}>
            {node.status}
          </span>
        )}

        {node.metadata && Object.keys(node.metadata).length > 0 && (
          <span className="text-xs text-secondary ml-auto">
            {Object.entries(node.metadata).map(([key, value]) => (
              <span key={key} className="ml-2">
                {key}: {String(value)}
              </span>
            ))}
          </span>
        )}
      </div>

      {hasChildren && isExpanded && (
        <div>
          {node.children!.map((child, idx) => (
            <TreeNodeComponent
              key={idx}
              node={child}
              level={level + 1}
              defaultExpandedLevels={defaultExpandedLevels}
              showIcons={showIcons}
              iconMap={iconMap}
            />
          ))}
        </div>
      )}
    </div>
  )
}

function getStatusColor(status: string): string {
  const statusLower = status.toLowerCase()
  switch (statusLower) {
    case 'healthy':
    case 'success':
      return 'bg-success/20 text-success border border-success/30'
    case 'warning':
      return 'bg-warning/20 text-warning border border-warning/30'
    case 'error':
    case 'critical':
      return 'bg-error/20 text-error border border-error/30'
    default:
      return 'bg-surface-variant text-secondary border border-primary-10'
  }
}
