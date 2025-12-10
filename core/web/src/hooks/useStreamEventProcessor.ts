/**
 * Stream Event Processor Hook
 *
 * Routes SSE events to specialized handlers and coordinates state updates.
 * Handles JSON parsing, validation, and event type dispatch.
 */

import { useCallback } from 'react'
import { useContextStore } from '@/stores/context.store'
import { useUIStateStore } from '@/stores/ui-state.store'
import { EventType } from '@/constants'
import { logger } from '@/lib/logger'
import {
  handleSnapshotEvent,
  handleContextStatsEvent,
  handleTaskEvent,
  handleTaskCreatedEvent,
  handleArtifactCreatedEvent,
  handleSkillLoadedEvent,
  handleMessageReceivedEvent,
  handleTaskCompletedEvent,
} from './utils/streamEventHandlers'
import type { BroadcastEvent } from '@/types/sse'
import { isCurrentAgentEvent, hasTaskInData, hasStepInData } from '@/types/sse'
import type { ExecutionStep } from '@/types/execution'
import { getStepTitle } from '@/types/execution'
import { extractFirstMessageText } from '@/utils/type-guards'

interface ContextStateEvent {
  type: EventType
  context_id?: string
  timestamp: string
  [key: string]: unknown
}

const parseEventData = (data: string): BroadcastEvent | null => {
  try {
    return JSON.parse(data)
  } catch (error) {
    logger.error('Failed to parse event data', error, 'useStreamEventProcessor')
    return null
  }
}

const createStateEvent = (type: EventType, event: BroadcastEvent): ContextStateEvent => ({
  type,
  context_id: event.context_id,
  timestamp: event.timestamp,
  ...event.data
})

export function useStreamEventProcessor() {
  const processEvent = useCallback((eventType: string | undefined, data: string) => {
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
            handleTaskCompletedEvent(event)
          }
          break
        }

        case EventType.TASK_CREATED:
        case 'task_created': {
          const event = parseEventData(data)
          if (event) {
            const taskInfo = hasTaskInData(event.data)
              ? { taskId: event.data.task.id, userMessage: extractFirstMessageText(event.data.task) }
              : { taskId: undefined, userMessage: undefined }

            console.log('[TASK_CREATED] User message received via SSE', {
              timestamp: new Date().toISOString(),
              contextId: event.context_id,
              ...taskInfo
            })
            logger.debug('Task created', { contextId: event.context_id }, 'useStreamEventProcessor')
            useContextStore.getState().handleStateEvent(createStateEvent(EventType.TASK_CREATED, event))
            handleTaskCreatedEvent(event)
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
            break
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

        case EventType.EXECUTION_STEP: {
          const event = parseEventData(data)
          if (event && hasStepInData(event.data)) {
            const stepData = event.data.step as ExecutionStep
            console.log('[EXECUTION_STEP] Step received via SSE', {
              timestamp: new Date().toISOString(),
              stepId: stepData.stepId,
              status: stepData.status,
              title: getStepTitle(stepData),
              contextId: event.context_id,
              taskId: stepData.taskId
            })
            logger.debug('Execution step received', { stepId: stepData.stepId, status: stepData.status, contextId: event.context_id }, 'useStreamEventProcessor')
            useUIStateStore.getState().addStep(stepData, event.context_id)
          }
          break
        }

        case EventType.HEARTBEAT:
          break

        case 'error':
          logger.error('Error event received', data, 'useStreamEventProcessor')
          break

        default:
          logger.debug('Unknown event type', { eventType }, 'useStreamEventProcessor')
      }
    } catch (error) {
      logger.error('Failed to process event', error, 'useStreamEventProcessor')
    }
  }, [])

  return { processEvent }
}
