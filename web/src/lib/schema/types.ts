/**
 * JSON Schema type definitions using official @types/json-schema
 */

import type { JSONSchema7, JSONSchema7Definition } from 'json-schema'

// Re-export official types
export type JsonSchema = JSONSchema7
export type JsonSchemaProperty = JSONSchema7Definition

/**
 * Data source configuration for dynamic dropdown population.
 * Used in x-data-source field to specify how to fetch options.
 */
export interface DataSourceConfig {
  /** Tool name to call for fetching data */
  tool: string
  /** Action parameter to pass to the tool */
  action: string
  /** Field in response items to use as option value */
  value_field: string
  /** Field in response items to use as option label (fallback) */
  label_field?: string
  /** Template for formatting labels with placeholders like {name} - {version} */
  label_template?: string
  /** Additional filter parameters to pass to the tool */
  filter?: Record<string, unknown>
  /** Cache full object in dropdown option for form population */
  cache_object?: boolean
}

/**
 * Extended JSON Schema with our custom x-data-source field
 */
export interface ExtendedJSONSchema7 extends JSONSchema7 {
  'x-data-source'?: DataSourceConfig
}

/**
 * Form field value types
 */
export type FieldValue = string | number | boolean | null | Record<string, unknown> | unknown[] | undefined

/**
 * Form values (key-value pairs)
 */
export type FormValues = Record<string, FieldValue>

/**
 * Validation error for a single field
 */
export interface ValidationError {
  field: string
  message: string
}

/**
 * Validation result
 */
export interface ValidationResult {
  valid: boolean
  errors: Record<string, string>
}

/**
 * Extract enum values from schema property
 */
export function getEnumValues(property: JSONSchema7): unknown[] {
  return property.enum || []
}

/**
 * Check if field is required
 */
export function isRequired(fieldName: string, schema: JSONSchema7): boolean {
  return Array.isArray(schema.required) && schema.required.includes(fieldName)
}

/**
 * Get field type (handles array of types)
 */
export function getFieldType(property: JSONSchema7): string {
  if (Array.isArray(property.type)) {
    // If multiple types, prefer first non-null type
    const nonNullType = property.type.find(t => t !== 'null')
    return nonNullType || property.type[0] || 'string'
  }
  return property.type || 'string'
}

/**
 * Check if property is an enum field
 */
export function isEnumField(property: JSONSchema7): boolean {
  return Array.isArray(property.enum) && property.enum.length > 0
}

/**
 * Get default value for a field
 */
export function getDefaultValue(property: JSONSchema7): FieldValue | undefined {
  return property.default as FieldValue | undefined
}

/**
 * Get a value from a nested object using dot notation path.
 *
 * @example
 * getNestedValue({card: {name: 'test'}}, 'card.name') // 'test'
 */
export function getNestedValue(obj: FormValues, path: string): FieldValue {
  const parts = path.split('.')
  let current: any = obj

  for (const part of parts) {
    if (current === undefined || current === null || typeof current !== 'object') {
      return undefined
    }
    current = current[part]
  }

  return current as FieldValue
}

/**
 * Set a value in a nested object using dot notation path.
 * Returns a new object (immutable update).
 *
 * @example
 * setNestedValue({}, 'card.name', 'test') // {card: {name: 'test'}}
 */
export function setNestedValue(obj: FormValues, path: string, value: FieldValue): FormValues {
  const parts = path.split('.')
  const result = { ...obj }
  let current: any = result

  // Navigate to the parent of the target property
  for (let i = 0; i < parts.length - 1; i++) {
    const part = parts[i]

    // Create or clone the nested object
    if (current[part] === undefined || typeof current[part] !== 'object' || Array.isArray(current[part])) {
      current[part] = {}
    } else {
      current[part] = { ...current[part] }
    }

    current = current[part]
  }

  // Set the value at the final key
  current[parts[parts.length - 1]] = value

  return result
}

/**
 * Flatten nested errors object to dot notation.
 *
 * @example
 * flattenErrors({card: {name: 'Required'}}) // {'card.name': 'Required'}
 */
export function flattenErrors(errors: any, prefix = ''): Record<string, string> {
  const flattened: Record<string, string> = {}

  for (const [key, value] of Object.entries(errors)) {
    const fullKey = prefix ? `${prefix}.${key}` : key

    if (typeof value === 'string') {
      flattened[fullKey] = value
    } else if (typeof value === 'object' && value !== null) {
      Object.assign(flattened, flattenErrors(value, fullKey))
    }
  }

  return flattened
}
