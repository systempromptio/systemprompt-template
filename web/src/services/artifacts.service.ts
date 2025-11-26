/**
 * Artifact Management Service
 *
 * Provides REST API operations for artifacts with built-in validation and
 * error handling. All artifacts are validated against schema before returning.
 *
 * @example
 * ```typescript
 * const { artifacts, error } = await artifactsService.listArtifacts(token)
 * if (error) { console.error(error); return }
 * artifacts?.forEach(a => console.log(a.id))
 * ```
 */

import type { Artifact as A2AArtifact } from '@a2a-js/sdk'
import type { Artifact } from '@/types/artifact'
import { toArtifact } from '@/types/artifact'
import { apiClient } from './api-client'
import { logger } from '@/lib/logger'

/**
 * Artifact service response with optional error
 */
interface ArtifactListResponse {
  artifacts?: Artifact[]
  error?: string
}

/**
 * Artifact service response for single artifact
 */
interface ArtifactResponse {
  artifact?: Artifact
  error?: string
}

/**
 * Service for artifact operations
 * @internal
 */
class ArtifactsService {
  /**
   * Validate and convert raw API artifacts to typed artifacts
   * @internal
   * @param artifacts - Raw artifacts from API
   * @returns Validated artifacts, skipping invalid ones
   */
  private validateArtifacts(artifacts: A2AArtifact[]): Artifact[] {
    return artifacts
      .map(artifact => {
        try {
          return toArtifact(artifact)
        } catch (e) {
          logger.warn('Skipping invalid artifact from API', e, 'artifacts-service')
          return null
        }
      })
      .filter((a): a is Artifact => a !== null)
  }

  /**
   * Fetch all artifacts with optional limit
   * @param authToken - JWT authorization token
   * @param limit - Optional limit on number of artifacts to return
   * @returns Artifacts list or error message
   *
   * @example
   * ```typescript
   * const { artifacts, error } = await artifactsService.listArtifacts(token, 50)
   * ```
   */
  async listArtifacts(
    authToken: string | null,
    limit?: number
  ): Promise<ArtifactListResponse> {
    const params = new URLSearchParams()
    if (limit) params.append('limit', limit.toString())

    const queryString = params.toString()
    const endpoint = queryString ? `/artifacts?${queryString}` : '/artifacts'

    const result = await apiClient.get<A2AArtifact[]>(endpoint, authToken)

    if (!result.data) {
      return { error: result.error }
    }

    const validatedArtifacts = this.validateArtifacts(result.data)
    return { artifacts: validatedArtifacts, error: result.error }
  }

  /**
   * Fetch artifacts for a specific context
   * @param contextId - UUID of context
   * @param authToken - JWT authorization token
   * @returns Artifacts for context or error message
   *
   * @example
   * ```typescript
   * const { artifacts } = await artifactsService.listArtifactsByContext(contextId, token)
   * ```
   */
  async listArtifactsByContext(
    contextId: string,
    authToken: string | null
  ): Promise<ArtifactListResponse> {
    const result = await apiClient.get<A2AArtifact[]>(
      `/contexts/${contextId}/artifacts`,
      authToken
    )

    if (!result.data) {
      return { error: result.error }
    }

    const validatedArtifacts = this.validateArtifacts(result.data)
    return { artifacts: validatedArtifacts, error: result.error }
  }

  /**
   * Fetch artifacts for a specific task
   * @param taskId - UUID of task
   * @param authToken - JWT authorization token
   * @returns Artifacts for task or error message
   *
   * @example
   * ```typescript
   * const { artifacts } = await artifactsService.listArtifactsByTask(taskId, token)
   * ```
   */
  async listArtifactsByTask(
    taskId: string,
    authToken: string | null
  ): Promise<ArtifactListResponse> {
    const result = await apiClient.get<A2AArtifact[]>(
      `/tasks/${taskId}/artifacts`,
      authToken
    )

    if (!result.data) {
      return { error: result.error }
    }

    const validatedArtifacts = this.validateArtifacts(result.data)
    return { artifacts: validatedArtifacts, error: result.error }
  }

  /**
   * Fetch single artifact by ID
   * @param artifactId - UUID of artifact
   * @param authToken - JWT authorization token
   * @returns Single artifact or error message
   *
   * @example
   * ```typescript
   * const { artifact, error } = await artifactsService.getArtifact(artifactId, token)
   * ```
   */
  async getArtifact(
    artifactId: string,
    authToken: string | null
  ): Promise<ArtifactResponse> {
    const result = await apiClient.get<A2AArtifact>(
      `/artifacts/${artifactId}`,
      authToken
    )

    if (!result.data) {
      return { error: result.error }
    }

    try {
      const validated = toArtifact(result.data)
      return { artifact: validated, error: result.error }
    } catch (e) {
      return { error: e instanceof Error ? e.message : 'Invalid artifact from API' }
    }
  }
}

/** Singleton instance of artifacts service */
export const artifactsService = new ArtifactsService()
