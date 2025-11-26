export interface BroadcastEvent {
  event_type: string
  context_id: string
  user_id: string
  data: any
  timestamp: string
}

export interface ArtifactCreatedEventData {
  artifact: any
  task_id?: string
  context_id: string
  [key: string]: any
}

export interface MessageReceivedEventData {
  context_id: string
  [key: string]: any
}

export interface TaskEventData {
  task: any
  context_id: string
  [key: string]: any
}

export interface CurrentAgentEvent {
  type: 'current_agent'
  context_id: string
  /** Agent name, or null to clear/remove agent assignment */
  agent_name: string | null
  timestamp: string
}

/**
 * Type guard to check if an event is a CurrentAgentEvent
 */
export function isCurrentAgentEvent(event: unknown): event is CurrentAgentEvent {
  return (
    typeof event === 'object' &&
    event !== null &&
    'type' in event &&
    event.type === 'current_agent' &&
    'context_id' in event &&
    typeof event.context_id === 'string' &&
    'agent_name' in event &&
    (typeof event.agent_name === 'string' || event.agent_name === null) &&
    'timestamp' in event &&
    typeof event.timestamp === 'string'
  )
}
