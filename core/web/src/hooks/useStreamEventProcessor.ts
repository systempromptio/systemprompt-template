/**
 * Stream Event Processor Hook
 *
 * Routes SSE events to specialized handlers and coordinates state updates.
 * Handles JSON parsing, validation, and event type dispatch.
 *
 * @example
 * ```typescript
 * const { processEvent } = useStreamEventProcessor()
 * processEvent('artifact_created', jsonString)
 * ```
 */

import { useCallback } from 'react'
import { useContextStore } from '@/stores/context.store'
import { EventType } from '@/constants'
import { logger } from '@/lib/logger'
import {
  handleSnapshotEvent,
  handleContextStatsEvent,
  handleTaskEvent,
  handleArtifactCreatedEvent,
  handleSkillLoadedEvent,
  handleMessageReceivedEvent,
  handleTaskCompletedEvent,
} from './utils/streamEventHandlers'
import type { BroadcastEvent } from '@/types/sse'
import { isCurrentAgentEvent } from '@/types/sse'

/**
 * Internal event structure for state updates
 */
interface ContextStateEvent {
  type: EventType
  context_id?: string
  timestamp: string
  [key: string]: unknown
}

/**
 * Safely parse JSON event data
 * @internal
 */
const parseEventData = (data: string): BroadcastEvent | null => {
  try {
    return JSON.parse(data)
  } catch (error) {
    logger.error('Failed to parse event data', error, 'useStreamEventProcessor')
    return null
  }
}

/**
 * Create context state event from broadcast event
 * @internal
 */
const createStateEvent = (type: EventType, event: BroadcastEvent): ContextStateEvent => ({
  type,
  context_id: event.context_id,
  timestamp: event.timestamp,
  ...event.data
})

/**
 * Hook for processing and routing SSE events.
 *
 * Routes different event types to specialized handlers and coordinates
 * store updates. Handles JSON parsing and event validation.
 *
 * Note: Uses store.getState() to avoid circular dependencies.
 * Never add store methods to useCallback dependency arrays.
 *
 * @returns Object with processEvent callback
 */
export function useStreamEventProcessor() {
  /**
   * Route and process incoming SSE event
   * @internal
   * @param eventType - The type of event received
   * @param data - JSON string containing event data
   */
  const processEvent = useCallback((eventType: string | undefined, data: string) => {
    console.log('[useStreamEventProcessor] Received event:', { eventType, dataPreview: data.substring(0, 200) })

    try {
      switch (eventType) {
        case 'snapshot':
          handleSnapshotEvent(data)
          break

        case 'context_stats':
          handleContextStatsEvent(data)
          break

        case EventType.ARTIFACT_CREATED: {
          const event = parseEventData(data)
          if (event) {
            useContextStore.getState().handleStateEvent(createStateEvent('artifact_created', event))
            handleArtifactCreatedEvent(event)
          }
          break
        }

        case EventType.SKILL_LOADED: {
          const event = parseEventData(data)
          if (event) {
            console.log('[DEBUG] useStreamEventProcessor - SKILL_LOADED event received:', {
              eventType,
              event,
              contextId: event?.context_id,
              data: event?.data
            })
            logger.debug('Skill loaded', { contextId: event.context_id }, 'useStreamEventProcessor')
            handleSkillLoadedEvent(event)
          }
          break
        }

        case 'message_received': {
          const event = parseEventData(data)
          if (event) {
            logger.debug('Message received', { contextId: event.context_id }, 'useStreamEventProcessor')
            useContextStore.getState().handleStateEvent(createStateEvent(EventType.MESSAGE_ADDED, event))
            handleMessageReceivedEvent(event)
          }
          break
        }

        case EventType.TASK_COMPLETED: {
          const event = parseEventData(data)
          if (event) {
            logger.debug('Task completed', { contextId: event.context_id }, 'useStreamEventProcessor')
            useContextStore.getState().handleStateEvent(createStateEvent(EventType.TASK_COMPLETED, event))
            handleTaskEvent(event)
            handleTaskCompletedEvent(event)
          }
          break
        }

        case EventType.TASK_CREATED: {
          const event = parseEventData(data)
          if (event) {
            logger.debug('Task created', { contextId: event.context_id }, 'useStreamEventProcessor')
            useContextStore.getState().handleStateEvent(createStateEvent(EventType.TASK_CREATED, event))
            handleTaskEvent(event)
          }
          break
        }

        case EventType.TASK_STATUS_CHANGED: {
          const event = parseEventData(data)
          if (event) {
            logger.debug('Task status changed', { contextId: event.context_id }, 'useStreamEventProcessor')
            useContextStore.getState().handleStateEvent(createStateEvent(EventType.TASK_STATUS_CHANGED, event))
            handleTaskEvent(event)
          }
          break
        }

        case EventType.CONTEXT_CREATED: {
          const event = parseEventData(data)
          if (event) {
            logger.debug('Context created', { contextId: event.context_id }, 'useStreamEventProcessor')
            useContextStore.getState().handleStateEvent(createStateEvent(EventType.CONTEXT_CREATED, event))
          }
          break
        }

        case EventType.CONTEXT_UPDATED: {
          const event = parseEventData(data)
          if (event) {
            logger.debug('Context updated', { contextId: event.context_id }, 'useStreamEventProcessor')
            useContextStore.getState().handleStateEvent(createStateEvent(EventType.CONTEXT_UPDATED, event))
          }
          break
        }

        case EventType.CONTEXT_DELETED: {
          const event = parseEventData(data)
          if (event) {
            logger.debug('Context deleted', { contextId: event.context_id }, 'useStreamEventProcessor')
            useContextStore.getState().handleStateEvent(createStateEvent(EventType.CONTEXT_DELETED, event))
          }
          break
        }

        case EventType.CURRENT_AGENT: {
          const event = parseEventData(data)

          if (!isCurrentAgentEvent(event)) {
            logger.error('Invalid current_agent event structure', { data }, 'useStreamEventProcessor')
            console.error('[useStreamEventProcessor] Invalid current_agent event:', event)
            break
          }

          console.log('[useStreamEventProcessor] Processing current_agent event:', event)

          if (event.agent_name === null) {
            console.warn('[useStreamEventProcessor] Clearing agent assignment', { contextId: event.context_id })
          } else {
            console.log('[useStreamEventProcessor] Setting agent:', { contextId: event.context_id, agentName: event.agent_name })
          }

          logger.debug('Current agent event', { contextId: event.context_id, agentName: event.agent_name }, 'useStreamEventProcessor')
          useContextStore.getState().handleStateEvent({
            type: EventType.CURRENT_AGENT,
            context_id: event.context_id,
            timestamp: event.timestamp,
            agent_name: event.agent_name
          })
          break
        }

        case EventType.HEARTBEAT:
          break

        case 'error':
          logger.error('Error event received', data, 'useStreamEventProcessor')
          break

        default:
          console.warn('[useStreamEventProcessor] Unknown event type:', eventType, 'Full data:', data)
          logger.debug('Unknown event type', { eventType }, 'useStreamEventProcessor')
      }
    } catch (error) {
      logger.error('Failed to process event', error, 'useStreamEventProcessor')
    }
  }, [])

  return { processEvent }
}
