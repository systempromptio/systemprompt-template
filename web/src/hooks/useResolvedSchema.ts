import { useMemo } from 'react'
import type { JSONSchema7 } from 'json-schema'
import type { FormValues } from '@/lib/schema/types'
import { resolveConditionalSchema } from '@/lib/schema/resolver'

/**
 * Hook to resolve conditional JSON Schema based on current form values.
 *
 * This ensures consistent schema resolution across rendering, validation, and submission.
 * The resolved schema includes conditional fields based on discriminator values (e.g., action).
 *
 * @param baseSchema - Base JSON Schema with conditional logic (allOf, if/then, etc.)
 * @param values - Current form values (used to resolve conditionals)
 * @returns Resolved schema with conditionals applied
 *
 * @example
 * ```tsx
 * const baseSchema = extractToolInputSchema(tool.inputSchema)
 * const resolvedSchema = useResolvedSchema(baseSchema, formValues)
 * // For action=delete, resolvedSchema includes uuid in properties and required
 * ```
 */
export function useResolvedSchema(
  baseSchema: JSONSchema7,
  values: FormValues
): JSONSchema7 {
  return useMemo(() => {
    return resolveConditionalSchema(baseSchema, values)
  }, [baseSchema, values])
}
