/**
 * Schema field router component.
 *
 * Analyzes JSON Schema and renders the appropriate field component
 * based on type, format, and other schema properties.
 *
 * @module tools/SchemaField
 */

import type { JSONSchema7 } from 'json-schema'
import type { FieldValue } from '@/lib/schema/types'
import { SchemaFieldRenderer } from './SchemaFieldRenderer'

export interface SchemaFieldProps {
  name: string
  property: JSONSchema7
  value: FieldValue
  onChange: (value: FieldValue) => void
  error?: string
  errors?: Record<string, string>
  required?: boolean
  onObjectSelect?: (obj: unknown) => void
  onLoadingChange?: (loading: boolean) => void
}

/**
 * Renders a form field based on JSON Schema property definition.
 *
 * Automatically selects the appropriate input type based on schema:
 * - Dynamic enum (x-data-source) → <DynamicEnumField>
 * - Static enum → <EnumField>
 * - string → <StringField> (text or textarea)
 * - number/integer → <NumberField>
 * - boolean → <BooleanField>
 * - object → <ObjectField> (nested)
 * - array → <ArrayField>
 *
 * @example
 * ```typescript
 * <SchemaField
 *   name="email"
 *   property={{ type: 'string', format: 'email' }}
 *   value={formData.email}
 *   onChange={(val) => updateField('email', val)}
 *   required
 * />
 * ```
 */
export const SchemaField = SchemaFieldRenderer


/**
 * Format field name for display (convert snake_case to Title Case).
 * Exported for use in field components.
 */
export function formatFieldName(name: string): string {
  return name
    .split('_')
    .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
    .join(' ')
}

/**
 * Format enum option for display.
 * Exported for use in field components.
 */
export function formatEnumOption(value: string): string {
  return value
    .split('_')
    .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
    .join(' ')
}
