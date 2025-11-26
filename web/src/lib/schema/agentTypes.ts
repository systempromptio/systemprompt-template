/**
 * Agent management response types with discriminated unions.
 *
 * All responses include a 'type' discriminator field to enable type-safe handling.
 */

export type AgentResponseType = 'confirmation' | 'success'
export type AgentAction = 'create' | 'read' | 'update' | 'delete'

export interface BaseResponse {
    type: AgentResponseType
    action: AgentAction
}

// ===== CREATE RESPONSES =====

export interface CreateConfirmation extends BaseResponse {
    type: 'confirmation'
    action: 'create'
    valid: boolean
    will_create: {
        name: string
        description: string
        version: string
        url: string
        port: number
        is_active: boolean
    }
    warnings?: string[]
}

export interface CreateSuccess extends BaseResponse {
    type: 'success'
    action: 'create'
    success: boolean
    uuid: string
    name: string
    url: string
    message: string
}

// ===== UPDATE RESPONSES =====

export interface UpdateConfirmation extends BaseResponse {
    type: 'confirmation'
    action: 'update'
    uuid: string
    current: Record<string, unknown>
    proposed: Record<string, unknown>
    changes: Record<string, {from: unknown; to: unknown}>
    warnings?: string[]
}

export interface UpdateSuccess extends BaseResponse {
    type: 'success'
    action: 'update'
    success: boolean
    uuid: string
    updated_fields: string[]
    message: string
}

// ===== DELETE RESPONSES =====

export interface DeleteConfirmation extends BaseResponse {
    type: 'confirmation'
    action: 'delete'
    uuid: string
    agent: {
        name: string
        version: string
        is_active: boolean
    }
    impact: {
        skills_lost: number
        sessions_terminated: number
        mcp_servers_detached: number
    }
    warnings?: string[]
}

export interface DeleteSuccess extends BaseResponse {
    type: 'success'
    action: 'delete'
    success: boolean
    uuid: string
    message: string
}

// ===== UNION TYPE =====

export type AgentResponse =
    | CreateConfirmation | CreateSuccess
    | UpdateConfirmation | UpdateSuccess
    | DeleteConfirmation | DeleteSuccess
