import type { Task } from '@/types/task'
import { apiClient } from './api-client'

class TasksService {

  async listTasksByContext(
    contextId: string,
    authToken: string | null
  ): Promise<{ tasks?: Task[]; error?: string }> {
    const result = await apiClient.get<Task[]>(
      `/contexts/${contextId}/tasks`,
      authToken
    )
    return { tasks: result.data, error: result.error }
  }

  async getTask(
    taskId: string,
    authToken: string | null
  ): Promise<{ task?: Task; error?: string }> {
    const result = await apiClient.get<Task>(
      `/tasks/${taskId}`,
      authToken
    )
    return { task: result.data, error: result.error }
  }

  async listTasks(
    authToken: string | null,
    status?: string,
    limit?: number
  ): Promise<{ tasks?: Task[]; error?: string }> {
    const params = new URLSearchParams()
    if (status) params.append('status', status)
    if (limit) params.append('limit', limit.toString())

    const queryString = params.toString()
    const endpoint = queryString ? `/tasks?${queryString}` : '/tasks'

    const result = await apiClient.get<Task[]>(endpoint, authToken)
    return { tasks: result.data, error: result.error }
  }
}

export const tasksService = new TasksService()
