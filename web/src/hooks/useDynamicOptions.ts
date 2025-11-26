import { useState, useEffect, useCallback } from 'react'
import type { DataSourceConfig } from '@/lib/schema/types'
import type { Artifact } from '@/types/artifact'
import { useToolsStore } from '@/stores/tools.store'
import { useMcpToolCaller } from './useMcpToolCaller'
import { useArtifactSubscription } from './useArtifactSubscription'
import { logger } from '@/lib/logger'

export interface DynamicOption {
  value: string
  label: string
  data?: unknown  // Cached full object when cache_object is true
}

export interface UseDynamicOptionsResult {
  options: DynamicOption[]
  loading: boolean
  error: string | null
  fetchFullObject: (uuid: string) => Promise<unknown | null>
}

/**
 * Hook to fetch dynamic dropdown options based on schema x-data-source configuration.
 *
 * Calls MCP tools to populate dropdown menus dynamically, supporting caching of
 * full objects and extracting options from various response formats.
 *
 * @param config - Data source configuration from schema
 * @returns Options array, loading state, and error
 *
 * @throws {Error} When tool execution fails
 *
 * @example
 * ```typescript
 * function SelectDatabase() {
 *   const dataSourceConfig = {
 *     tool_name: 'list_databases',
 *     tool_server: 'db-server'
 *   }
 *
 *   const { options, loading, error } = useDynamicOptions(dataSourceConfig)
 *
 *   return (
 *     <select disabled={loading}>
 *       <option>Select database...</option>
 *       {options.map(opt => (
 *         <option key={opt.value} value={opt.value}>{opt.label}</option>
 *       ))}
 *     </select>
 *   )
 * }
 * ```
 */
export function useDynamicOptions(config: DataSourceConfig | undefined): UseDynamicOptionsResult {
  const [options, setOptions] = useState<DynamicOption[]>([])
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const tools = useToolsStore((state) => state.tools)
  const { callTool } = useMcpToolCaller()

  const handleArtifact = useCallback((artifact: Artifact) => {
    try {
      const artifactData = extractDataFromArtifact(artifact)
      const items = extractItems(artifactData)
      const opts = items.map((item: unknown) => {
        if (typeof item !== 'object' || item === null) return { value: '', label: '' }
        const record = item as Record<string, unknown>
        const valueField = config?.value_field ?? ''
        const labelField = config?.label_field ?? ''
        return ({
          value: String(record[valueField] ?? ''),
          label: String(record[labelField] ?? '')
        })
      })
      setOptions(opts)
      setLoading(false)
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Failed to extract options'
      setError(message)
      setLoading(false)
    }
  }, [config])

  const handleTimeout = useCallback(() => {
    setError('Tool execution timed out')
    setLoading(false)
  }, [])

  const { subscribe } = useArtifactSubscription({
    onArtifact: handleArtifact,
    onTimeout: handleTimeout,
    timeout: 30000
  })

  useEffect(() => {
    if (!config) {
      setOptions([])
      return
    }

    const fetchOptions = async () => {
      setLoading(true)
      setError(null)

      try {
        // Find the tool in the tools store
        const tool = tools.find((t) => t.name === config.tool)
        if (!tool) {
          throw new Error(`Tool '${config.tool}' not found`)
        }

        logger.debug('Fetching dynamic options', { tool: config.tool }, 'useDynamicOptions')

        const executionId = crypto.randomUUID()
        subscribe(executionId)

        await callTool(
          tool.serverEndpoint,
          tool.name,
          {
            action: config.action,
            ...config.filter
          }
        )
      } catch (err) {
        logger.error('Error fetching options', err, 'useDynamicOptions')
        const message = err instanceof Error ? err.message : 'Failed to fetch options'
        setError(message)
        setLoading(false)
      }
    }

    fetchOptions()
  }, [config, tools, callTool, subscribe])

  /**
   * Fetch FULL object with all data (e.g., agent with skills loaded from MCP servers)
   *
   * This calls the same tool but with the specific UUID to get the complete object,
   * not the lightweight list version.
   */

  const fetchFullObject = useCallback(async (uuid: string): Promise<unknown | null> => {
    if (!config) return null

    try {
      const tool = tools.find((t) => t.name === config.tool)
      if (!tool) {
        throw new Error(`Tool '${config.tool}' not found`)
      }

      logger.debug('Fetching full object', { tool: config.tool }, 'useDynamicOptions')

      const executionId = crypto.randomUUID()
      const artifactPromise = new Promise<unknown>((resolve, reject) => {
        const timeout = setTimeout(() => reject(new Error('Timeout')), 30000)

        const unsubscribeProxy = useArtifactSubscription({
          onArtifact: (artifact) => {
            clearTimeout(timeout)
            resolve(extractDataFromArtifact(artifact))
          },
          onTimeout: () => {
            clearTimeout(timeout)
            reject(new Error('Tool execution timed out'))
          }
        })

        unsubscribeProxy.subscribe(executionId)
      })

      await callTool(
        tool.serverEndpoint,
        tool.name,
        { action: config.action, uuid }
      )

      return await artifactPromise
    } catch (err) {
      logger.error('Error fetching full object', err, 'useDynamicOptions')
      return null
    }
  }, [config, tools, callTool])

  return { options, loading, error, fetchFullObject }
}

/**
 * Type guard to check if a value has a specific property.
 */
function hasProperty<K extends string>(
  obj: unknown,
  key: K
): obj is Record<K, unknown> {
  return typeof obj === 'object' && obj !== null && key in obj
}

/**
 * Extract data from A2A artifact with type safety.
 * Artifacts can have data directly or in parts array.
 */
function extractDataFromArtifact(artifact: unknown): unknown {
  if (!artifact || typeof artifact !== 'object') {
    return null
  }

  // Check direct data property (A2A v1 style)
  if (hasProperty(artifact, 'data')) {
    return artifact.data
  }

  // Check parts array for data part (A2A v2 style)
  if (hasProperty(artifact, 'parts') && Array.isArray(artifact.parts)) {
    for (const part of artifact.parts) {
      if (typeof part === 'object' && part !== null && hasProperty(part, 'kind')) {
        if (part.kind === 'data' && hasProperty(part, 'data')) {
          return part.data
        }
      }
    }
  }

  // Fallback: return whole artifact
  return artifact
}

/**
 * Extract items array from response data with type safety.
 * Handles common response structures: direct array, {items}, {data}, {results}
 */
function extractItems(data: unknown): unknown[] {
  // Direct array
  if (Array.isArray(data)) {
    return data
  }

  // Object with items/data/results property
  if (typeof data === 'object' && data !== null) {
    if (hasProperty(data, 'items') && Array.isArray(data.items)) {
      return data.items
    }
    if (hasProperty(data, 'data') && Array.isArray(data.data)) {
      return data.data
    }
    if (hasProperty(data, 'results') && Array.isArray(data.results)) {
      return data.results
    }
  }

  return []
}

