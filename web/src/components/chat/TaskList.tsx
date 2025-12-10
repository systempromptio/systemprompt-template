import { useEffect, useRef, useMemo } from 'react'
import { TaskView } from './TaskView'
import { StreamingResponseBubble } from './StreamingResponseBubble'
import { useAgentStore } from '@/stores/agent.store'
import { useUIStateStore } from '@/stores/ui-state.store'
import { Avatar } from '@/components/ui/Avatar'
import { Quote, LogIn } from 'lucide-react'
import { useAuth } from '@/hooks/useAuth'
import type { Task } from '@/types/task'
import type { ExecutionStep } from '@/types/execution'

interface TaskListProps {
  tasks: Task[]
  contextId: string
}

export function TaskList({ tasks, contextId }: TaskListProps) {
  const bottomRef = useRef<HTMLDivElement>(null)
  const selectedAgent = useAgentStore((state) => state.selectedAgent)
  const { isRealUser, showLogin } = useAuth()

  const activeStreamingTaskId = useUIStateStore((s) => s.activeStreamingTaskId)
  const stepIdsByTask = useUIStateStore((s) => s.stepIdsByTask)
  const stepsById = useUIStateStore((s) => s.stepsById)

  const streamingState = useMemo(() => {
    console.log('%c[TASKLIST] Computing streamingState', 'color: #00ccff;', {
      timestamp: new Date().toISOString(),
      taskCount: tasks.length,
      activeStreamingTaskId,
      lastTaskId: tasks[tasks.length - 1]?.id,
      stepsForLastTask: tasks.length > 0 ? stepIdsByTask[tasks[tasks.length - 1].id]?.length || 0 : 0
    })

    if (tasks.length === 0) {
      console.log('[TASKLIST] No tasks, returning null')
      return null
    }

    const lastTask = tasks[tasks.length - 1]
    const isStreaming = activeStreamingTaskId === lastTask.id

    console.log('[TASKLIST] Streaming check', {
      lastTaskId: lastTask.id,
      activeStreamingTaskId,
      isStreaming
    })

    if (!isStreaming) {
      console.log('[TASKLIST] Not streaming, returning null')
      return null
    }

    const stepIds = stepIdsByTask[lastTask.id] || []
    const steps = stepIds
      .map((id) => stepsById[id])
      .filter((step): step is ExecutionStep => !!step)
      .sort((a, b) => new Date(a.startedAt).getTime() - new Date(b.startedAt).getTime())

    console.log('[TASKLIST] Returning streamingState with', {
      taskId: lastTask.id,
      stepCount: steps.length
    })

    return { taskId: lastTask.id, executionSteps: steps }
  }, [tasks, activeStreamingTaskId, stepIdsByTask, stepsById])

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: 'smooth', block: 'nearest' })
  }, [tasks, streamingState])

  if (tasks.length === 0) {
    const agentName = selectedAgent?.name
    const agentDescription = selectedAgent?.description || 'Ready to assist you'

    return (
      <div className="flex-1 flex items-center justify-center text-text-secondary px-md py-sm md:py-xl">
        <div className="text-center max-w-2xl w-full">
          <div className="flex items-center justify-center gap-sm mb-md">
            <Avatar
              variant="agent"
              agentName={agentName}
              agentId={selectedAgent?.url}
              size="md"
              showGlow={true}
              animated={true}
            />
            <span className="text-xl font-heading font-medium uppercase tracking-wide text-primary">
              {agentName}
            </span>
          </div>
          <div className="relative mb-md px-8 md:px-0">
            <Quote className="absolute left-0 md:-left-8 top-0 w-5 h-5 md:w-6 md:h-6 text-primary/30" />
            <p className="text-sm leading-relaxed text-white font-body">
              {agentDescription}
            </p>
          </div>
          {isRealUser ? (
            <p className="text-sm">Say hello, or choose another agent from the menu</p>
          ) : (
            <button
              onClick={() => showLogin()}
              className="inline-flex items-center gap-xs px-md py-sm rounded-lg bg-primary/10 hover:bg-primary/20 border border-primary/30 hover:border-primary/60 text-primary hover:text-primary-dark transition-all duration-fast hover:scale-105"
            >
              <LogIn className="w-4 h-4" />
              <span className="text-sm font-body">Sign in to start chatting with {agentName}</span>
            </button>
          )}
        </div>
      </div>
    )
  }

  return (
    <div className="flex-1 overflow-y-auto overflow-x-hidden px-md py-md space-y-md scrollbar-thin max-w-full">
      {tasks.map((task) => (
        <TaskView
          key={task.id}
          task={task}
          contextId={contextId}
        />
      ))}

      {streamingState && (
        <StreamingResponseBubble
          executionSteps={streamingState.executionSteps}
          agentName={selectedAgent?.name}
          agentId={selectedAgent?.url}
        />
      )}

      <div ref={bottomRef} />
    </div>
  )
}
