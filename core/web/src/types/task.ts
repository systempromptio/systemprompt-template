import type { Task as A2ATask } from '@a2a-js/sdk'
import type { ExecutionStep } from './execution'

export interface TaskMetadata {
  task_type: 'mcp_execution' | 'agent_message'
  agent_name: string
  tool_name?: string
  mcp_server_name?: string
  created_at: string
  updated_at?: string
  started_at?: string
  completed_at?: string
  execution_time_ms?: number
  input_tokens?: number
  output_tokens?: number
  model?: string
  executionSteps?: ExecutionStep[]
  [k: string]: unknown
}

export type Task = Omit<A2ATask, 'metadata'> & {
  metadata: TaskMetadata
}

export function validateTask(task: A2ATask): task is Task {
  if (!task.metadata) {
    return false
  }

  const metadata = task.metadata as Record<string, unknown>
  const validTaskTypes = ['mcp_execution', 'agent_message']
  return (
    typeof metadata.task_type === 'string' &&
    validTaskTypes.includes(metadata.task_type) &&
    typeof metadata.agent_name === 'string' &&
    typeof metadata.created_at === 'string'
  )
}

export function toTask(task: A2ATask): Task {
  if (!validateTask(task)) {
    throw new Error(
      `Invalid task: missing required metadata fields. ` +
      `Expected: task_type (mcp_execution | agent_message), agent_name, created_at. ` +
      `Received: ${JSON.stringify(task.metadata)}`
    )
  }
  return task
}
