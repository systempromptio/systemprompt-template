import { useChatStore, type ChatStore } from '@/stores/chat.store'
import { Clock, CheckCircle, XCircle, AlertCircle, Loader } from 'lucide-react'
import { cn } from '@/lib/utils/cn'
import type { Task } from '@a2a-js/sdk'

export function TaskMonitor() {
  const { tasksById, taskIds } = useChatStore((state: ChatStore) => ({
    tasksById: state.tasksById,
    taskIds: state.taskIds
  }))

  const taskList = taskIds.map(id => tasksById[id]).filter(Boolean) as Task[]

  if (taskList.length === 0) {
    return (
      <div className="text-center py-8 text-gray-400">
        <div className="text-sm">No active tasks</div>
      </div>
    )
  }

  return (
    <div className="space-y-4">
      <h3 className="text-sm font-semibold text-gray-700 uppercase">Tasks</h3>
      <div className="space-y-2">
        {taskList.map((task) => (
          <TaskItem key={task.id} task={task} />
        ))}
      </div>
    </div>
  )
}

interface TaskItemProps {
  task: Task
}

function TaskItem({ task }: TaskItemProps) {
  const state = task.status?.state || 'unknown'

  return (
    <div className="p-3 bg-white border rounded-lg">
      <div className="flex items-start gap-2">
        {getStatusIcon(state)}
        <div className="flex-1 min-w-0">
          <div className="text-sm font-medium truncate">
            Task {task.id.slice(0, 8)}...
          </div>
          <div className="text-xs text-gray-500 capitalize">{state}</div>
          {task.status?.message && (
            <div className="text-xs text-gray-600 mt-1 line-clamp-2">
              {typeof task.status.message === 'string'
                ? task.status.message
                : task.status.message.parts?.[0]?.kind === 'text'
                  ? task.status.message.parts[0].text
                  : ''}
            </div>
          )}
        </div>
      </div>
    </div>
  )
}

function getStatusIcon(state: string) {
  const iconClass = 'w-4 h-4'

  switch (state) {
    case 'submitted':
      return <Clock className={cn(iconClass, 'text-gray-400')} />
    case 'working':
      return <Loader className={cn(iconClass, 'text-blue-500 animate-spin')} />
    case 'input-required':
      return <AlertCircle className={cn(iconClass, 'text-yellow-500')} />
    case 'completed':
      return <CheckCircle className={cn(iconClass, 'text-green-500')} />
    case 'failed':
    case 'rejected':
      return <XCircle className={cn(iconClass, 'text-red-500')} />
    case 'canceled':
      return <XCircle className={cn(iconClass, 'text-gray-400')} />
    default:
      return <Clock className={cn(iconClass, 'text-gray-400')} />
  }
}