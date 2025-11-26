import { useState, useCallback } from 'react'
import type { McpTool } from '@/stores/tools.store'
import type { FormValues } from '@/lib/schema/types'
import { canAutoSubmit, extractDefaults } from '@/lib/schema/defaults'
import { extractToolInputSchema } from '@/lib/schema/validation'
import { useMcpToolCaller } from './useMcpToolCaller'
import { logger } from '@/lib/logger'

type RenderBehavior = 'modal' | 'inline' | 'silent' | 'both' | undefined

function isRenderBehavior(value: unknown): value is RenderBehavior {
  return value === undefined || ['modal', 'inline', 'silent', 'both'].includes(value as string)
}

function extractRenderBehavior(outputSchema: unknown): RenderBehavior {
  if (typeof outputSchema === 'object' && outputSchema !== null && 'x-render-behavior' in outputSchema) {
    const behavior = (outputSchema as Record<string, unknown>)['x-render-behavior']
    return isRenderBehavior(behavior) ? behavior : undefined
  }
  return undefined
}

/**
 * Hook for managing tool parameter collection and execution.
 *
 * This hook decides whether to:
 * 1. Show parameter modal (if tool has required params without defaults)
 * 2. Auto-execute with defaults (if all required params have defaults)
 *
 * Flow:
 * - User clicks tool → executeTool()
 * - Check if canAutoSubmit(schema)
 * - Yes → Call tool immediately with defaults
 * - No → Show modal → User fills form → submitParameters()
 *
 * @returns Hook state and execution methods
 *
 * @throws {Error} When tool execution fails
 *
 * @example
 * ```typescript
 * function ToolDialog() {
 *   const {
 *     showModal,
 *     selectedTool,
 *     executeTool,
 *     submitParameters,
 *     closeModal
 *   } = useToolParameters()
 *
 *   return (
 *     <>
 *       <button onClick={() => executeTool(myTool)}>
 *         {myTool.name}
 *       </button>
 *       {showModal && (
 *         <ParameterModal
 *           tool={selectedTool}
 *           onSubmit={submitParameters}
 *           onClose={closeModal}
 *         />
 *       )}
 *     </>
 *   )
 * }
 * ```
 */
export function useToolParameters() {
  const [showModal, setShowModal] = useState(false)
  const [selectedTool, setSelectedTool] = useState<McpTool | null>(null)
  const { callTool } = useMcpToolCaller()

  /**
   * Execute a tool, either immediately or after collecting parameters
   */
  const executeTool = useCallback(
    async (tool: McpTool) => {
      logger.debug('Executing tool', { tool: tool.name }, 'useToolParameters')

      // Safely extract and validate the input schema
      const schema = extractToolInputSchema(tool.inputSchema, tool.name)

      // Check if we can auto-submit (no required fields or all have defaults)
      if (canAutoSubmit(schema)) {
        logger.debug('Auto-submitting with defaults', { tool: tool.name }, 'useToolParameters')
        const defaults = extractDefaults(schema)
        const renderBehavior = extractRenderBehavior(tool.outputSchema)
        await callTool(
          tool.serverEndpoint,
          tool.name,
          defaults,
          tool.serverName,
          renderBehavior
        )
        return
      }

      // Need user input - show modal
      setSelectedTool(tool)
      setShowModal(true)
    },
    [callTool]
  )

  /**
   * Submit parameters from modal
   */
  const submitParameters = useCallback(
    async (tool: McpTool, parameters: FormValues) => {
      logger.debug('Submitting parameters', { tool: tool.name }, 'useToolParameters')
      const renderBehavior = extractRenderBehavior(tool.outputSchema)
      await callTool(
        tool.serverEndpoint,
        tool.name,
        parameters,
        tool.serverName,
        renderBehavior
      )
    },
    [callTool]
  )

  /**
   * Close parameter modal
   */
  const closeModal = useCallback(() => {
    setShowModal(false)
    setSelectedTool(null)
  }, [])

  return {
    executeTool,
    submitParameters,
    closeModal,
    showModal,
    selectedTool,
  }
}
