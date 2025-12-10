import { useState } from 'react'
import {
  CheckCircle,
  Circle,
  AlertCircle,
  Loader,
  Clock,
  ChevronDown,
  ChevronRight,
  Code,
  FileText,
  Wrench,
} from 'lucide-react'
import { getStepTypeIcon, getStepTypeLabel } from '@/constants'
import { cn } from '@/lib/utils/cn'
import type { ExecutionStep } from '@/types/execution'
import {
  getToolName,
  getToolArguments,
  getToolResult,
  getReasoning,
  getStepTitle,
  getStepSubtitle,
} from '@/types/execution'

interface ExecutionStepCardProps {
  step: ExecutionStep
  stepNumber?: number
  isExpanded?: boolean
  onToggle?: () => void
}

function formatDuration(ms: number): string {
  if (ms < 1000) {
    return `${ms}ms`
  }
  const seconds = ms / 1000
  if (seconds < 60) {
    return `${seconds.toFixed(1)}s`
  }
  const mins = Math.floor(seconds / 60)
  const secs = Math.floor(seconds % 60)
  return `${mins}m ${secs}s`
}

function formatJson(data: unknown): string {
  try {
    return JSON.stringify(data, null, 2)
  } catch {
    return String(data)
  }
}

interface CollapsibleJsonProps {
  label: string
  icon: React.ReactNode
  data: unknown
  defaultOpen?: boolean
}

function CollapsibleJson({ label, icon, data, defaultOpen = false }: CollapsibleJsonProps) {
  const [isOpen, setIsOpen] = useState(defaultOpen)
  const formattedJson = formatJson(data)
  const lineCount = formattedJson.split('\n').length

  return (
    <div className="border border-border rounded-lg overflow-hidden">
      <button
        onClick={(e) => {
          e.stopPropagation()
          setIsOpen(!isOpen)
        }}
        className="w-full flex items-center gap-2 px-3 py-2 bg-muted/50 hover:bg-muted transition-colors text-left"
      >
        {isOpen ? <ChevronDown className="w-3.5 h-3.5 text-text-tertiary" /> : <ChevronRight className="w-3.5 h-3.5 text-text-tertiary" />}
        {icon}
        <span className="text-xs font-medium text-text-secondary flex-1">{label}</span>
        <span className="text-[10px] text-text-tertiary">{lineCount} lines</span>
      </button>
      {isOpen && (
        <pre className="p-3 text-xs font-mono bg-surface overflow-x-auto max-h-64 whitespace-pre-wrap break-all">
          {formattedJson}
        </pre>
      )}
    </div>
  )
}

export function ExecutionStepCard({
  step,
  stepNumber,
  isExpanded = false,
  onToggle,
}: ExecutionStepCardProps) {
  const toolName = getToolName(step)
  const toolArguments = getToolArguments(step)
  const toolResult = getToolResult(step)
  const reasoning = getReasoning(step)
  const subtitle = getStepSubtitle(step)

  const getStatusIcon = () => {
    switch (step.status) {
      case 'completed':
        return <CheckCircle className="w-4 h-4 text-success" />
      case 'in_progress':
        return <Loader className="w-4 h-4 text-primary animate-spin" />
      case 'failed':
        return <AlertCircle className="w-4 h-4 text-error" />
      default:
        return <Circle className="w-4 h-4 text-text-tertiary" />
    }
  }

  const duration =
    step.durationMs ??
    (step.completedAt && step.startedAt
      ? new Date(step.completedAt).getTime() - new Date(step.startedAt).getTime()
      : null)

  const isFailed = step.status === 'failed'
  const isCompleted = step.status === 'completed'
  const isInProgress = step.status === 'in_progress'

  const hasToolData = toolName || toolArguments || toolResult
  const hasReasoning = !!reasoning

  return (
    <div
      className={cn(
        'rounded-lg border transition-all cursor-pointer',
        isFailed ? 'bg-error/5 border-error/30' : 'bg-surface border-border',
        isExpanded && 'ring-2 ring-primary/30',
        isInProgress && 'border-primary/50'
      )}
      onClick={onToggle}
    >
      {/* Header */}
      <div className="flex items-start gap-3 p-3">
        <div className={cn(
          'flex-shrink-0 mt-0.5',
          isFailed ? 'text-error' : isCompleted ? 'text-success' : isInProgress ? 'text-primary' : 'text-text-tertiary'
        )}>
          {getStatusIcon()}
        </div>
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2 mb-1 flex-wrap">
            {stepNumber !== undefined && (
              <span className="inline-flex items-center justify-center w-5 h-5 rounded-full bg-primary/10 text-[10px] font-bold text-primary">
                {stepNumber}
              </span>
            )}
            <span className="inline-flex items-center gap-1 text-xs font-medium text-text-secondary">
              {getStepTypeIcon(step.content.type)}
              {getStepTypeLabel(step.content.type)}
            </span>
            {duration !== null && duration > 0 && (
              <span className="flex items-center gap-1 px-1.5 py-0.5 rounded bg-muted text-xs tabular-nums ml-auto text-text-tertiary">
                <Clock className="w-3 h-3" />
                {formatDuration(duration)}
              </span>
            )}
          </div>
          <p className={cn('text-sm font-medium', isExpanded ? '' : 'line-clamp-1')}>{getStepTitle(step)}</p>

          {subtitle && (
            <p className="text-xs text-text-secondary mt-1 italic">{subtitle}</p>
          )}

          {/* Preview when collapsed */}
          {!isExpanded && toolResult && (
            <p className="text-xs text-text-tertiary mt-1 line-clamp-1 font-mono">
              {typeof toolResult === 'string' ? toolResult : JSON.stringify(toolResult).slice(0, 80)}...
            </p>
          )}

          {isCompleted && step.completedAt && !isExpanded && (
            <p className="text-[10px] text-text-tertiary mt-1">
              Completed at {new Date(step.completedAt).toLocaleTimeString()}
            </p>
          )}
        </div>
      </div>

      {/* Expanded Content */}
      {isExpanded && (
        <div className="px-3 pb-3 space-y-3 border-t border-border pt-3 mx-3 mb-0">
          {/* Timestamps */}
          <div className="flex gap-4 text-[10px] text-text-tertiary">
            <span>Started: {new Date(step.startedAt).toLocaleString()}</span>
            {step.completedAt && (
              <span>Completed: {new Date(step.completedAt).toLocaleString()}</span>
            )}
          </div>

          {/* Reasoning Section */}
          {hasReasoning && (
            <div>
              <span className="text-xs font-medium text-text-secondary flex items-center gap-1 mb-1">
                <FileText className="w-3.5 h-3.5" />
                Reasoning
              </span>
              <p className="text-xs text-foreground/80 whitespace-pre-wrap bg-muted/30 p-2 rounded">
                {reasoning}
              </p>
            </div>
          )}

          {/* Tool Section */}
          {hasToolData && (
            <div className="space-y-2">
              {toolName && (
                <div className="flex items-center gap-2">
                  <span className="text-xs font-medium text-text-secondary flex items-center gap-1">
                    <Wrench className="w-3.5 h-3.5" />
                    Tool:
                  </span>
                  <code className="text-xs font-mono bg-primary/10 text-primary px-1.5 py-0.5 rounded">
                    {toolName}
                  </code>
                </div>
              )}

              {toolArguments && (
                <CollapsibleJson
                  label="Input Arguments"
                  icon={<Code className="w-3.5 h-3.5 text-text-tertiary" />}
                  data={toolArguments}
                  defaultOpen={false}
                />
              )}

              {toolResult && (
                <CollapsibleJson
                  label="Output Result"
                  icon={<Code className="w-3.5 h-3.5 text-text-tertiary" />}
                  data={toolResult}
                  defaultOpen={true}
                />
              )}
            </div>
          )}

          {/* Error Section */}
          {isFailed && step.errorMessage && (
            <div className="p-3 bg-error/10 border border-error/30 rounded-lg">
              <span className="text-xs font-medium text-error flex items-center gap-1 mb-1">
                <AlertCircle className="w-3.5 h-3.5" />
                Error
              </span>
              <p className="text-xs text-error/90 whitespace-pre-wrap">{step.errorMessage}</p>
            </div>
          )}
        </div>
      )}
    </div>
  )
}
