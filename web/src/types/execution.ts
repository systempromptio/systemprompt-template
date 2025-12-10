export type StepType =
  | 'understanding'
  | 'planning'
  | 'skill_usage'
  | 'tool_execution'
  | 'completion'

export type StepStatus = 'pending' | 'in_progress' | 'completed' | 'failed'

export interface PlannedTool {
  tool_name: string
  arguments: unknown
}

export interface StepContent {
  type: StepType
  reasoning?: string
  planned_tools?: PlannedTool[]
  skill_id?: string
  skill_name?: string
  tool_name?: string
  tool_arguments?: unknown
  tool_result?: unknown
}

export interface ExecutionStep {
  stepId: string
  taskId: string
  status: StepStatus
  startedAt: string
  completedAt?: string
  durationMs?: number
  errorMessage?: string
  content: StepContent
}

function formatToolName(name: string): string {
  return name
    .replace(/_/g, ' ')
    .replace(/\b\w/g, c => c.toUpperCase())
}

export function getStepTitle(step: ExecutionStep): string {
  switch (step.content.type) {
    case 'understanding':
      return 'ANALYZING REQUEST...'
    case 'planning':
      return 'PLANNING RESPONSE...'
    case 'skill_usage':
      return `USING ${formatToolName(step.content.skill_name || 'SKILL')}...`
    case 'tool_execution':
      return `RUNNING ${formatToolName(step.content.tool_name || 'TOOL')}...`
    case 'completion':
      return 'COMPLETE'
  }
}

export function getStepSubtitle(step: ExecutionStep): string | undefined {
  if (step.content.type === 'tool_execution' && step.content.tool_arguments) {
    const args = step.content.tool_arguments as Record<string, unknown>
    const keys = Object.keys(args).slice(0, 2)
    if (keys.length > 0) {
      return keys.map(k => `${k}: ${String(args[k]).slice(0, 20)}`).join(', ')
    }
  }
  return undefined
}

export function getStepType(step: ExecutionStep): StepType {
  return step.content.type
}

export function getToolName(step: ExecutionStep): string | undefined {
  if (step.content.type === 'tool_execution') return step.content.tool_name
  if (step.content.type === 'skill_usage') return step.content.skill_name
  return undefined
}

export function getSkillId(step: ExecutionStep): string | undefined {
  if (step.content.type === 'skill_usage') return step.content.skill_id
  return undefined
}

export function getToolArguments(step: ExecutionStep): Record<string, unknown> | undefined {
  if (step.content.type === 'tool_execution') return step.content.tool_arguments as Record<string, unknown> | undefined
  return undefined
}

export function getToolResult(step: ExecutionStep): Record<string, unknown> | undefined {
  if (step.content.type === 'tool_execution') return step.content.tool_result as Record<string, unknown> | undefined
  return undefined
}

export function getReasoning(step: ExecutionStep): string | undefined {
  if (step.content.type === 'planning') return step.content.reasoning
  return undefined
}

export function getPlannedTools(step: ExecutionStep): PlannedTool[] | undefined {
  if (step.content.type === 'planning') return step.content.planned_tools
  return undefined
}

export function getStepIcon(step: ExecutionStep): string {
  const iconMap: Record<StepType, string> = {
    understanding: 'brain',
    planning: 'map',
    skill_usage: 'sparkles',
    tool_execution: 'wrench',
    completion: 'check',
  }
  return iconMap[step.content.type]
}

export interface ExecutionStepEvent {
  kind: 'execution_step'
  taskId: string
  contextId: string
  step: ExecutionStep
}

export function isExecutionStepEvent(data: unknown): data is ExecutionStepEvent {
  const event = data as Record<string, unknown>
  return (
    event.kind === 'execution_step' &&
    typeof event.taskId === 'string' &&
    typeof event.contextId === 'string' &&
    typeof event.step === 'object' &&
    event.step !== null
  )
}
