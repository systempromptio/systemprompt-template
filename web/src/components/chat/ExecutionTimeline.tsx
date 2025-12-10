import { useState, useEffect, useRef, type ReactElement } from 'react'
import {
  Loader,
  ChevronDown,
  ChevronUp,
  AlertCircle,
  ChevronRight,
  Code,
  Wrench,
  Sparkles,
  Brain,
  Lightbulb,
  MapPin,
  Check,
} from 'lucide-react'
import { cn } from '@/lib/utils/cn'
import { useExpandableList } from '@/hooks/useExpandableList'
import type { ExecutionStep, StepStatus, StepType } from '@/types/execution'
import { getToolName, getToolArguments, getToolResult, getReasoning, getPlannedTools, getStepTitle, getStepSubtitle, getSkillId } from '@/types/execution'

const morphingStyles = `
@keyframes morph {
  0%, 100% {
    border-radius: 50%;
    transform: rotate(0deg) scale(1);
  }
  25% {
    border-radius: 30%;
    transform: rotate(90deg) scale(0.9);
  }
  50% {
    border-radius: 50% 0 50% 0;
    transform: rotate(180deg) scale(1);
  }
  75% {
    border-radius: 30%;
    transform: rotate(270deg) scale(0.9);
  }
}
@keyframes colorShift {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.7; }
}
`

function MorphingShape() {
  return (
    <>
      <style>{morphingStyles}</style>
      <div
        className="w-6 h-6 bg-primary"
        style={{
          animation: 'morph 3s ease-in-out infinite, colorShift 2s ease-in-out infinite',
        }}
      />
    </>
  )
}

type StreamingProps = {
  mode: 'streaming'
  steps: ExecutionStep[]
  variant?: 'standalone' | 'bubble'
  className?: string
}

type StaticProps = {
  mode: 'static'
  steps: ExecutionStep[]
  initialCollapsed?: boolean
  className?: string
}

type ModalProps = {
  mode: 'modal'
  steps: ExecutionStep[]
  className?: string
}

export type ExecutionTimelineProps = StreamingProps | StaticProps | ModalProps

function formatDuration(ms: number): string {
  if (ms < 1000) return `${ms}ms`
  const seconds = ms / 1000
  if (seconds < 60) return `${seconds.toFixed(1)}s`
  const mins = Math.floor(seconds / 60)
  const secs = Math.floor(seconds % 60)
  return `${mins}m ${secs}s`
}

function formatElapsedTime(startedAt: string): string {
  const elapsed = (Date.now() - new Date(startedAt).getTime()) / 1000
  if (elapsed < 1) return '0s'
  if (elapsed < 60) return `${Math.floor(elapsed)}s`
  return `${Math.floor(elapsed / 60)}m ${Math.floor(elapsed % 60)}s`
}

function useElapsedTime(startedAt: string | undefined, isRunning: boolean): string {
  const [elapsed, setElapsed] = useState('')
  useEffect(() => {
    if (!startedAt || !isRunning) { setElapsed(''); return }
    const update = () => setElapsed(formatElapsedTime(startedAt))
    update()
    const interval = setInterval(update, 1000)
    return () => clearInterval(interval)
  }, [startedAt, isRunning])
  return elapsed
}

function getStepIcon(stepType: StepType): ReactElement {
  const iconClass = "w-3 h-3"
  switch (stepType) {
    case 'understanding': return <Brain className={iconClass} />
    case 'planning': return <MapPin className={iconClass} />
    case 'skill_usage': return <Sparkles className={iconClass} />
    case 'tool_execution': return <Wrench className={iconClass} />
    case 'completion': return <Check className={iconClass} />
  }
}

function getStatusColor(status: StepStatus): string {
  switch (status) {
    case 'completed': return 'bg-emerald-500 text-white'
    case 'in_progress': return 'bg-blue-500 text-white'
    case 'failed': return 'bg-red-500 text-white'
    default: return 'bg-gray-400 text-white'
  }
}

function getStatusBorderColor(status: StepStatus): string {
  switch (status) {
    case 'completed': return 'border-emerald-500'
    case 'in_progress': return 'border-blue-500'
    case 'failed': return 'border-red-500'
    default: return 'border-gray-400'
  }
}

function StreamingView({ steps, className }: { steps: ExecutionStep[], variant?: 'standalone' | 'bubble', className?: string }) {
  const currentStep = steps.find(s => s.status === 'in_progress') ?? steps[steps.length - 1]
  const completed = steps.filter(s => s.status === 'completed').length
  const hasFailed = steps.some(s => s.status === 'failed')

  const isStreaming = true
  const elapsed = useElapsedTime(currentStep?.startedAt, isStreaming)

  const activeStep = currentStep ?? steps[steps.length - 1]
  const toolName = activeStep ? getToolName(activeStep) : undefined
  const toolArgs = activeStep ? getToolArguments(activeStep) : undefined
  const reasoning = activeStep ? getReasoning(activeStep) : undefined
  const plannedTools = activeStep ? getPlannedTools(activeStep) : undefined
  const subtitle = activeStep ? getStepSubtitle(activeStep) : undefined

  const stepProgress = steps.length > 0 ? `${completed}/${steps.length}` : '0/0'
  const title = currentStep ? getStepTitle(currentStep) : 'PROCESSING REQUEST...'

  return (
    <div className={cn('space-y-3', className)}>
      <div className="flex items-start gap-3">
        <div className="relative flex-shrink-0">
          <div className={cn(
            'w-10 h-10 rounded-full flex items-center justify-center transition-colors duration-300',
            hasFailed ? 'bg-error/20' : 'bg-primary/20'
          )}>
            {hasFailed ? (
              <AlertCircle className="w-6 h-6 text-error" />
            ) : (
              <MorphingShape />
            )}
          </div>
        </div>

        <div className="flex-1 min-w-0">
          <h4 className="text-sm font-medium text-text-primary leading-tight uppercase transition-all duration-200">
            {title}
          </h4>
          <div className="flex items-center gap-1.5 text-xs text-text-tertiary mt-0.5">
            <span className="tabular-nums uppercase">STEP {stepProgress}</span>
            {elapsed && (
              <>
                <span className="text-text-muted">•</span>
                <span className="tabular-nums">{elapsed}</span>
              </>
            )}
          </div>
        </div>
      </div>

      {steps.length > 0 && (
        <div className="flex items-center px-1">
          {steps.map((step, i) => {
            const stepDuration = step.durationMs ?? (step.completedAt && step.startedAt
              ? new Date(step.completedAt).getTime() - new Date(step.startedAt).getTime()
              : null)

            return (
              <div key={step.stepId} className="flex items-center group relative">
                <div
                  className={cn(
                    'w-5 h-5 rounded-full flex items-center justify-center transition-all text-[10px]',
                    getStatusColor(step.status),
                    step.status === 'in_progress' && 'ring-2 ring-primary/40 ring-offset-1 scale-110'
                  )}
                >
                  {step.status === 'in_progress' ? (
                    <Loader className="w-2.5 h-2.5 animate-spin" />
                  ) : step.status === 'completed' ? (
                    <Check className="w-2.5 h-2.5" />
                  ) : step.status === 'failed' ? (
                    <AlertCircle className="w-2.5 h-2.5" />
                  ) : (
                    <span className="text-[8px] font-bold">{i + 1}</span>
                  )}
                </div>

                <div className="absolute bottom-full left-1/2 -translate-x-1/2 mb-2 hidden group-hover:block z-10">
                  <div className="bg-surface border border-border rounded-md shadow-lg p-2 whitespace-nowrap min-w-[120px]">
                    <div className="text-xs font-medium uppercase">{getStepTitle(step)}</div>
                    {stepDuration !== null && stepDuration > 0 && (
                      <div className="text-[10px] text-text-tertiary mt-0.5">
                        DURATION: {formatDuration(stepDuration)}
                      </div>
                    )}
                    {getStepSubtitle(step) && (
                      <div className="text-[10px] text-text-tertiary mt-0.5 uppercase">{getStepSubtitle(step)}</div>
                    )}
                  </div>
                </div>

                {i < steps.length - 1 && (
                  <div className={cn(
                    'w-3 h-0.5',
                    step.status === 'completed' ? 'bg-success' : 'bg-muted'
                  )} />
                )}
              </div>
            )
          })}

          <div className="flex items-center gap-1 ml-2">
            <div className="w-2 h-2 rounded-full bg-muted animate-pulse" />
            <div className="w-2 h-2 rounded-full bg-muted/60 animate-pulse" style={{ animationDelay: '150ms' }} />
            <div className="w-2 h-2 rounded-full bg-muted/30 animate-pulse" style={{ animationDelay: '300ms' }} />
          </div>
        </div>
      )}

      {(toolName || reasoning || plannedTools || subtitle || activeStep?.content.type === 'tool_execution') && (
        <div className="bg-muted/30 rounded-md p-2 border border-border/50">
          <div className="flex items-start gap-2">
            <div className="text-text-tertiary mt-0.5">
              {toolName || activeStep?.content.type === 'tool_execution' ? <Wrench className="w-3.5 h-3.5" /> : <Lightbulb className="w-3.5 h-3.5" />}
            </div>
            <div className="flex-1 min-w-0">
              {toolName && (
                <div className="flex items-center gap-2">
                  <code className="text-xs font-mono bg-primary/10 text-primary px-1.5 py-0.5 rounded uppercase">
                    {toolName.replace(/_/g, ' ')}
                  </code>
                </div>
              )}
              {toolArgs && Object.keys(toolArgs).length > 0 && (
                <div className="mt-1.5">
                  <ExpandableToolArgs args={toolArgs as Record<string, unknown>} />
                </div>
              )}
              {reasoning && (
                <p className="text-xs text-text-secondary mt-1 line-clamp-2 uppercase">{reasoning}</p>
              )}
              {plannedTools && plannedTools.length > 0 && (
                <div className="mt-2 space-y-1">
                  <div className="text-[10px] text-text-tertiary uppercase font-medium">PLANNED TOOLS ({plannedTools.length})</div>
                  <div className="flex flex-wrap gap-1">
                    {plannedTools.map((tool, i) => (
                      <code key={i} className="text-[10px] font-mono bg-secondary/10 text-secondary px-1.5 py-0.5 rounded uppercase">
                        {tool.tool_name.replace(/_/g, ' ')}
                      </code>
                    ))}
                  </div>
                </div>
              )}
              {activeStep?.content.type === 'tool_execution' && !toolArgs && (
                <p className="text-xs text-text-tertiary mt-1 uppercase">EXECUTING...</p>
              )}
              {activeStep?.content.type === 'tool_execution' && toolArgs && Object.keys(toolArgs).length === 0 && (
                <p className="text-xs text-text-tertiary mt-1 uppercase">NO PARAMETERS</p>
              )}
              {!reasoning && !toolName && !plannedTools && subtitle && (
                <p className="text-xs text-text-secondary mt-1 uppercase">{subtitle}</p>
              )}
            </div>
          </div>
        </div>
      )}
    </div>
  )
}

function StaticView({ steps, initialCollapsed = false, className }: { steps: ExecutionStep[], initialCollapsed?: boolean, className?: string }) {
  const [isCollapsed, setIsCollapsed] = useState(initialCollapsed)
  const completed = steps.filter(s => s.status === 'completed').length
  const hasFailed = steps.some(s => s.status === 'failed')

  return (
    <div className={cn('border-t border-border bg-muted/30', className)}>
      <button
        onClick={() => setIsCollapsed(!isCollapsed)}
        className="w-full flex items-center justify-between px-3 py-2.5 hover:bg-muted/50 transition-colors"
      >
        <div className="flex items-center gap-2">
          <h4 className="text-sm font-semibold text-text-primary">Execution Steps</h4>
          <span className={cn(
            'text-[11px] px-1.5 py-0.5 rounded-full tabular-nums',
            hasFailed ? 'bg-error/10 text-error' : 'bg-success/10 text-success'
          )}>
            {completed}/{steps.length}
          </span>
        </div>
        {isCollapsed ? <ChevronDown className="w-4 h-4 text-text-tertiary" /> : <ChevronUp className="w-4 h-4 text-text-tertiary" />}
      </button>
      {!isCollapsed && (
        <div className="px-3 pb-3">
          <ModalStepList steps={steps} />
        </div>
      )}
    </div>
  )
}

function ModalStepList({ steps }: { steps: ExecutionStep[] }) {
  const { isExpanded, toggle } = useExpandableList()
  const completed = steps.filter(s => s.status === 'completed').length
  const hasFailed = steps.some(s => s.status === 'failed')

  const totalDuration = steps.reduce((acc, step) => {
    const stepDuration = step.durationMs ?? (step.completedAt && step.startedAt
      ? new Date(step.completedAt).getTime() - new Date(step.startedAt).getTime()
      : 0)
    return acc + stepDuration
  }, 0)

  return (
    <div className="space-y-2">
      <div className="flex items-center gap-2 mb-3">
        <span className={cn(
          'text-[11px] px-1.5 py-0.5 rounded-full tabular-nums font-medium',
          hasFailed ? 'bg-error/10 text-error' : 'bg-muted text-text-secondary'
        )}>
          {completed}/{steps.length} COMPLETED
        </span>
        {totalDuration > 0 && (
          <span className="text-[11px] px-1.5 py-0.5 rounded-full tabular-nums font-medium bg-muted text-text-secondary">
            TOTAL: {formatDuration(totalDuration)}
          </span>
        )}
      </div>
      {steps.map((step, index) => (
        <ModalStepCard key={step.stepId} step={step} index={index} isExpanded={isExpanded(step.stepId)} onToggle={() => toggle(step.stepId)} />
      ))}
    </div>
  )
}

function ModalStepCard({ step, index, isExpanded, onToggle }: { step: ExecutionStep, index: number, isExpanded: boolean, onToggle: () => void }) {
  const toolName = getToolName(step)
  const toolArgs = getToolArguments(step)
  const toolResult = getToolResult(step)
  const reasoning = getReasoning(step)
  const plannedTools = getPlannedTools(step)
  const skillId = getSkillId(step)
  const duration = step.durationMs ?? (step.completedAt && step.startedAt ? new Date(step.completedAt).getTime() - new Date(step.startedAt).getTime() : null)

  const hasDetail = toolName || toolArgs || toolResult || reasoning || plannedTools || step.errorMessage || skillId ||
    step.content.type === 'understanding' || step.content.type === 'planning' || step.content.type === 'completion'


  return (
    <div
      className={cn(
        'border rounded-lg overflow-hidden transition-all',
        getStatusBorderColor(step.status),
        step.status === 'failed' && 'bg-error/5',
        step.status === 'in_progress' && 'bg-primary/5',
        isExpanded && 'ring-1 ring-primary/20'
      )}
    >
      <button
        onClick={onToggle}
        className="w-full flex items-start gap-2.5 p-3 text-left hover:bg-muted/30 transition-colors"
      >
        <div className={cn('w-5 h-5 rounded-full flex items-center justify-center flex-shrink-0 mt-0.5', getStatusColor(step.status))}>
          {step.status === 'in_progress' ? (
            <Loader className="w-2.5 h-2.5 animate-spin" />
          ) : step.status === 'completed' ? (
            <Check className="w-2.5 h-2.5" />
          ) : step.status === 'failed' ? (
            <AlertCircle className="w-2.5 h-2.5" />
          ) : (
            <span className="text-[9px] font-bold">{index + 1}</span>
          )}
        </div>

        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-1.5 mb-0.5">
            <span className="text-text-tertiary flex-shrink-0">{getStepIcon(step.content.type)}</span>
            <h4 className="text-sm font-medium text-text-primary truncate leading-tight uppercase">{getStepTitle(step)}</h4>
          </div>
          {getStepSubtitle(step) && (
            <p className="text-xs text-text-tertiary truncate leading-snug uppercase">{getStepSubtitle(step)}</p>
          )}
          {duration !== null && duration > 0 && (
            <p className="text-[10px] text-text-muted tabular-nums mt-1">{formatDuration(duration)}</p>
          )}
        </div>

        {hasDetail && (
          <ChevronRight className={cn('w-4 h-4 text-text-tertiary transition-transform flex-shrink-0 mt-0.5', isExpanded && 'rotate-90')} />
        )}
      </button>

      {isExpanded && (
        <div className="border-t border-border bg-surface p-3 space-y-3">
          <div className="flex gap-4 text-[10px] text-text-tertiary">
            <span>Started: {new Date(step.startedAt).toLocaleTimeString()}</span>
            {step.completedAt && <span>Completed: {new Date(step.completedAt).toLocaleTimeString()}</span>}
          </div>

          {step.content.type === 'understanding' && (
            <DetailSection icon={<Brain className="w-3.5 h-3.5" />} label="Status">
              <p className="text-xs text-text-secondary">REQUEST RECEIVED AND PARSED</p>
            </DetailSection>
          )}

          {step.content.type === 'planning' && !reasoning && (
            <DetailSection icon={<MapPin className="w-3.5 h-3.5" />} label="Status">
              <p className="text-xs text-text-secondary">DETERMINING EXECUTION STRATEGY</p>
            </DetailSection>
          )}

          {reasoning && (
            <DetailSection icon={<Lightbulb className="w-3.5 h-3.5" />} label="Reasoning">
              <p className="text-xs whitespace-pre-wrap">{reasoning}</p>
            </DetailSection>
          )}

          {plannedTools && plannedTools.length > 0 && (
            <DetailSection icon={<Wrench className="w-3.5 h-3.5" />} label={`Planned Tools (${plannedTools.length})`} collapsible defaultOpen>
              <div className="space-y-2">
                {plannedTools.map((tool, i) => (
                  <div key={i} className="border border-border/50 rounded p-2">
                    <code className="text-xs font-mono bg-secondary/10 text-secondary px-1.5 py-0.5 rounded uppercase">
                      {tool.tool_name.replace(/_/g, ' ')}
                    </code>
                    {tool.arguments && typeof tool.arguments === 'object' && Object.keys(tool.arguments as Record<string, unknown>).length > 0 ? (
                      <div className="mt-2">
                        <ExpandableJson data={tool.arguments as Record<string, unknown>} />
                      </div>
                    ) : null}
                  </div>
                ))}
              </div>
            </DetailSection>
          )}

          {step.content.type === 'completion' && (
            <DetailSection icon={<Check className="w-3.5 h-3.5" />} label="Status">
              <p className="text-xs text-text-secondary">TASK EXECUTION COMPLETED SUCCESSFULLY</p>
            </DetailSection>
          )}

          {toolName && (
            <DetailSection icon={<Wrench className="w-3.5 h-3.5" />} label="Tool">
              <code className="text-xs font-mono bg-primary/10 text-primary px-1.5 py-0.5 rounded uppercase">{toolName.replace(/_/g, ' ')}</code>
            </DetailSection>
          )}

          {skillId && (
            <DetailSection icon={<Sparkles className="w-3.5 h-3.5" />} label="Skill ID">
              <code className="text-xs font-mono bg-secondary/10 text-secondary px-1.5 py-0.5 rounded">{skillId}</code>
            </DetailSection>
          )}

          {toolArgs && Object.keys(toolArgs).length > 0 && (
            <DetailSection icon={<Code className="w-3.5 h-3.5" />} label="Input" collapsible defaultOpen={false}>
              <ExpandableJson data={toolArgs} />
            </DetailSection>
          )}

          {toolResult && (
            <DetailSection icon={<Code className="w-3.5 h-3.5" />} label="Output" collapsible defaultOpen>
              <ExpandableJson data={toolResult} />
            </DetailSection>
          )}

          {step.errorMessage && (
            <div className="p-2 bg-error/10 border border-error/30 rounded">
              <div className="flex items-center gap-1.5 text-error text-xs font-medium mb-1">
                <AlertCircle className="w-3.5 h-3.5" />
                Error
              </div>
              <p className="text-xs text-error/90 whitespace-pre-wrap">{step.errorMessage}</p>
            </div>
          )}
        </div>
      )}
    </div>
  )
}

function DetailSection({ icon, label, children, collapsible = false, defaultOpen = true }: {
  icon: ReactElement
  label: string
  children: React.ReactNode
  collapsible?: boolean
  defaultOpen?: boolean
}) {
  const [isOpen, setIsOpen] = useState(defaultOpen)

  if (!collapsible) {
    return (
      <div>
        <div className="flex items-center gap-1.5 text-text-secondary text-[11px] font-semibold uppercase tracking-wide mb-1.5">
          <span className="text-text-tertiary">{icon}</span>
          {label}
        </div>
        <div className="pl-5">{children}</div>
      </div>
    )
  }

  return (
    <div className="border border-border rounded overflow-hidden">
      <button
        onClick={() => setIsOpen(!isOpen)}
        className="w-full flex items-center gap-1.5 px-2.5 py-2 bg-muted/30 hover:bg-muted/50 transition-colors text-left"
      >
        <ChevronRight className={cn('w-3 h-3 text-text-tertiary transition-transform', isOpen && 'rotate-90')} />
        <span className="text-text-tertiary">{icon}</span>
        <span className="text-[11px] font-semibold uppercase tracking-wide text-text-secondary">{label}</span>
      </button>
      {isOpen && <div className="p-2.5 bg-surface">{children}</div>}
    </div>
  )
}

function ExpandableJson({ data }: { data: unknown }) {
  const [isExpanded, setIsExpanded] = useState(false)
  const contentRef = useRef<HTMLPreElement>(null)
  const [needsExpand, setNeedsExpand] = useState(false)
  const json = typeof data === 'string' ? data : JSON.stringify(data, null, 2)

  useEffect(() => {
    if (contentRef.current) {
      setNeedsExpand(contentRef.current.scrollHeight > 96)
    }
  }, [json])

  return (
    <div className="relative">
      <pre
        ref={contentRef}
        className={cn(
          'text-[11px] font-mono bg-muted/30 p-2 rounded overflow-auto whitespace-pre-wrap cursor-pointer transition-all duration-200',
          isExpanded ? 'max-h-[600px]' : 'max-h-24'
        )}
        onClick={() => setIsExpanded(!isExpanded)}
      >
        {json}
      </pre>
      {!isExpanded && needsExpand && (
        <div className="absolute bottom-0 left-0 right-0 h-8 bg-gradient-to-t from-muted/90 via-muted/60 to-transparent pointer-events-none flex items-end justify-center pb-1 rounded-b">
          <span className="text-[9px] text-text-tertiary uppercase tracking-wide">Click to expand</span>
        </div>
      )}
      {isExpanded && needsExpand && (
        <div className="absolute bottom-0 left-0 right-0 flex justify-center pb-1 pointer-events-none">
          <span className="text-[9px] text-text-tertiary uppercase tracking-wide pointer-events-auto cursor-pointer hover:text-text-secondary" onClick={(e) => { e.stopPropagation(); setIsExpanded(false); }}>Click to collapse</span>
        </div>
      )}
    </div>
  )
}

function ExpandableToolArgs({ args }: { args: Record<string, unknown> }) {
  const [isExpanded, setIsExpanded] = useState(false)
  const entries = Object.entries(args)
  const hasMore = entries.length > 3

  const formatValue = (value: unknown, expanded: boolean): string => {
    if (typeof value === 'string') {
      if (expanded) return `"${value}"`
      return value.length > 50 ? `"${value.slice(0, 50)}..."` : `"${value}"`
    }
    if (typeof value === 'boolean' || typeof value === 'number') {
      return String(value)
    }
    if (Array.isArray(value)) {
      if (expanded) return JSON.stringify(value, null, 2)
      return `[${value.length} items]`
    }
    if (value && typeof value === 'object') {
      if (expanded) return JSON.stringify(value, null, 2)
      return '{...}'
    }
    return String(value)
  }

  const displayEntries = isExpanded ? entries : entries.slice(0, 3)

  return (
    <div
      className={cn(
        'text-[11px] font-mono text-text-tertiary bg-surface/50 rounded px-2 py-1 cursor-pointer transition-all duration-200',
        isExpanded ? 'max-h-[400px] overflow-auto' : 'max-h-20 overflow-hidden'
      )}
      onClick={() => setIsExpanded(!isExpanded)}
    >
      <div className="space-y-0.5">
        {displayEntries.map(([key, value]) => (
          <div key={key} className={isExpanded ? 'whitespace-pre-wrap' : 'truncate'}>
            <span className="text-primary/70">{key}:</span>{' '}
            <span className="text-text-secondary">{formatValue(value, isExpanded)}</span>
          </div>
        ))}
        {!isExpanded && hasMore && (
          <div className="text-text-tertiary">+ {entries.length - 3} more... <span className="text-[9px]">(click to expand)</span></div>
        )}
        {isExpanded && hasMore && (
          <div className="text-text-tertiary text-[9px] mt-1">(click to collapse)</div>
        )}
      </div>
    </div>
  )
}

function ModalView({ steps, className }: { steps: ExecutionStep[], className?: string }) {
  return (
    <div className={className}>
      <ModalStepList steps={steps} />
    </div>
  )
}

export function ExecutionTimeline(props: ExecutionTimelineProps): ReactElement | null {
  const { steps, className } = props

  if (steps.length === 0 && props.mode !== 'streaming') {
    return null
  }

  switch (props.mode) {
    case 'streaming':
      return <StreamingView steps={steps} variant={props.variant} className={className} />
    case 'static':
      return <StaticView steps={steps} initialCollapsed={props.initialCollapsed} className={className} />
    case 'modal':
      return <ModalView steps={steps} className={className} />
  }
}
