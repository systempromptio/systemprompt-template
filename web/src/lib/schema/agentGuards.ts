/**
 * Type guards for agent management responses.
 *
 * These guards enable type-safe discrimination of response types at runtime.
 */

import type {
    AgentResponse,
    CreateConfirmation,
    UpdateConfirmation,
    DeleteConfirmation,
    CreateSuccess,
    UpdateSuccess,
    DeleteSuccess
} from './agentTypes'

// ===== GENERAL GUARDS =====

export function isConfirmation(response: AgentResponse): response is
    CreateConfirmation | UpdateConfirmation | DeleteConfirmation {
    return response.type === 'confirmation'
}

export function isSuccess(response: AgentResponse): response is
    CreateSuccess | UpdateSuccess | DeleteSuccess {
    return response.type === 'success'
}

// ===== SPECIFIC CONFIRMATION GUARDS =====

export function isCreateConfirmation(response: AgentResponse): response is CreateConfirmation {
    return response.type === 'confirmation' && response.action === 'create'
}

export function isUpdateConfirmation(response: AgentResponse): response is UpdateConfirmation {
    return response.type === 'confirmation' && response.action === 'update'
}

export function isDeleteConfirmation(response: AgentResponse): response is DeleteConfirmation {
    return response.type === 'confirmation' && response.action === 'delete'
}

// ===== SPECIFIC SUCCESS GUARDS =====

export function isCreateSuccess(response: AgentResponse): response is CreateSuccess {
    return response.type === 'success' && response.action === 'create'
}

export function isUpdateSuccess(response: AgentResponse): response is UpdateSuccess {
    return response.type === 'success' && response.action === 'update'
}

export function isDeleteSuccess(response: AgentResponse): response is DeleteSuccess {
    return response.type === 'success' && response.action === 'delete'
}

// ===== HELPER TO CHECK IF RESPONSE IS AGENT RESPONSE =====

export function isAgentResponse(data: unknown): data is AgentResponse {
    if (typeof data !== 'object' || data === null) {
        return false
    }

    const response = data as Record<string, unknown>

    return (
        ('type' in response && (response.type === 'confirmation' || response.type === 'success')) &&
        ('action' in response && typeof response.action === 'string')
    )
}
