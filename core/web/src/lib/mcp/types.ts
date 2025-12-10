import type { CallToolResult } from '@modelcontextprotocol/sdk/types.js'
import type {
  ArtifactType,
  TableHints,
  ChartHints,
  CodeHints,
  FormHints,
  TreeHints,
  PresentationHints,
} from '@/types/artifact'

export interface McpArrayResponse<T = unknown> {
  items: T[]
  count?: number
  query_params?: Record<string, unknown>
}

export interface ChartData {
  labels: string[]
  datasets: Array<{
    label?: string
    data: number[]
    color?: string
  }>
}

export interface TreeNode {
  name: string
  status?: string
  children?: TreeNode[]
  metadata?: Record<string, unknown>
  [key: string]: unknown
}

export interface McpOutputSchema {
  type: string
  description?: string
  properties?: Record<string, JsonSchemaProperty>
  required?: string[]
  items?: McpOutputSchema
  oneOf?: McpOutputSchema[]
  'x-artifact-type'?: ArtifactType
  'x-table-hints'?: TableHints
  'x-chart-hints'?: ChartHints
  'x-code-hints'?: CodeHints
  'x-form-hints'?: FormHints
  'x-tree-hints'?: TreeHints
  'x-presentation-hints'?: PresentationHints
  'x-rendering-hints'?: Record<string, unknown>
  [key: string]: unknown
}

export interface JsonSchemaProperty {
  type: string
  description?: string
  enum?: string[]
  items?: JsonSchemaProperty
  properties?: Record<string, JsonSchemaProperty>
}

export interface ValidationError {
  path: string[]
  message: string
  expected?: string
  received?: string
}

export function hasStructuredContent(
  result: CallToolResult
): result is CallToolResult & { structuredContent: Record<string, unknown> } {
  return result.structuredContent !== undefined
}

export function isArrayResponse(data: unknown): data is McpArrayResponse {
  return (
    typeof data === 'object' &&
    data !== null &&
    'items' in data &&
    Array.isArray((data as McpArrayResponse).items)
  )
}

export function hasLabelsAndDatasets(data: unknown): data is ChartData {
  return (
    typeof data === 'object' &&
    data !== null &&
    'labels' in data &&
    'datasets' in data &&
    Array.isArray((data as ChartData).labels) &&
    Array.isArray((data as ChartData).datasets)
  )
}

export function hasNameProperty(data: unknown): data is TreeNode {
  return (
    typeof data === 'object' &&
    data !== null &&
    'name' in data &&
    typeof (data as TreeNode).name === 'string'
  )
}

export function isTableResult(
  result: CallToolResult,
  schema?: McpOutputSchema
): result is CallToolResult & { structuredContent: McpArrayResponse } {
  return (
    schema?.['x-artifact-type'] === 'table' &&
    hasStructuredContent(result) &&
    isArrayResponse(result.structuredContent)
  )
}

export function isChartResult(
  result: CallToolResult,
  schema?: McpOutputSchema
): result is CallToolResult & { structuredContent: ChartData } {
  return (
    schema?.['x-artifact-type'] === 'chart' &&
    hasStructuredContent(result) &&
    hasLabelsAndDatasets(result.structuredContent)
  )
}

export function isTreeResult(
  result: CallToolResult,
  schema?: McpOutputSchema
): result is CallToolResult & { structuredContent: TreeNode } {
  return (
    schema?.['x-artifact-type'] === 'tree' &&
    hasStructuredContent(result) &&
    hasNameProperty(result.structuredContent)
  )
}

export function isCodeResult(
  result: CallToolResult,
  schema?: McpOutputSchema
): result is CallToolResult & { structuredContent: string } {
  return (
    schema?.['x-artifact-type'] === 'code' &&
    hasStructuredContent(result) &&
    typeof result.structuredContent === 'string'
  )
}

export type { CallToolResult } from '@modelcontextprotocol/sdk/types.js'

// =============================================================================
// Row Data Guards - For table row data validation
// =============================================================================

/**
 * Guard for table row with context_id field.
 *
 * @example
 * ```typescript
 * // Before:
 * const contextId = row['context_id'] as string
 *
 * // After:
 * if (!hasContextId(row)) {
 *   throw new Error('Row missing context_id')
 * }
 * const contextId = row.context_id  // TypeScript knows it's string
 * ```
 */
export function hasContextId(row: unknown): row is Record<string, unknown> & { context_id: string } {
  return (
    typeof row === 'object' &&
    row !== null &&
    'context_id' in row &&
    typeof (row as Record<string, unknown>).context_id === 'string'
  )
}

/**
 * Guard for row with specific required fields.
 *
 * @example
 * ```typescript
 * if (hasRequiredFields(row, ['id', 'name', 'email'])) {
 *   // row.id, row.name, row.email all exist (unknown type)
 * }
 * ```
 */
export function hasRequiredFields<K extends string>(
  row: unknown,
  fields: K[]
): row is Record<string, unknown> & Record<K, unknown> {
  if (typeof row !== 'object' || row === null) return false
  return fields.every(field => field in row)
}

/**
 * Guard for validation_errors in metadata.
 */
export function hasValidationErrors(
  metadata: unknown
): metadata is { validation_errors: string[] } {
  if (typeof metadata !== 'object' || metadata === null) return false
  if (!('validation_errors' in metadata)) return false
  const errors = (metadata as Record<string, unknown>).validation_errors
  return Array.isArray(errors) && errors.every(e => typeof e === 'string')
}
