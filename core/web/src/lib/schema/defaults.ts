import type { JSONSchema7 } from 'json-schema'
import type { FormValues, FieldValue } from './types'
import { getFieldType } from './types'

/**
 * Extracts default values from a JSON Schema.
 *
 * This function recursively walks through a JSON Schema and extracts
 * all default values defined in the schema. This is useful for pre-filling
 * forms with sensible defaults.
 *
 * @param schema - JSON Schema to extract defaults from
 * @returns Object containing default values for all fields with defaults
 */
export function extractDefaults(schema: JSONSchema7): FormValues {
  const defaults: FormValues = {}

  if (!schema.properties) return defaults

  for (const [fieldName, property] of Object.entries(schema.properties)) {
    if (typeof property === 'boolean') continue
    const defaultValue = getDefaultForProperty(property)
    if (defaultValue !== undefined) {
      defaults[fieldName] = defaultValue
    }
  }

  return defaults
}

/**
 * Checks if a property has an explicit default value in the schema.
 *
 * This is used to distinguish between:
 * - Explicit defaults: "default": 7 in schema
 * - Implicit defaults: first enum value, minimum value, etc.
 *
 * @param property - JSON Schema property
 * @returns true if property has explicit default
 */
function hasExplicitDefault(property: JSONSchema7): boolean {
  return property.default !== undefined
}

/**
 * Get default value for a single schema property
 */
function getDefaultForProperty(property: JSONSchema7): FieldValue | undefined {
  // If explicit default is defined, use it
  if (property.default !== undefined) {
    return property.default as FieldValue
  }

  // Otherwise, return implicit defaults based on type
  const fieldType = getFieldType(property)

  switch (fieldType) {
    case 'string':
      // If enum, use first value as default
      if (property.enum && property.enum.length > 0) {
        return property.enum[0] as string
      }
      return undefined

    case 'number':
    case 'integer':
      // If has minimum, use that as default
      if (property.minimum !== undefined) {
        return property.minimum
      }
      return undefined

    case 'boolean':
      // Boolean defaults to false if not specified
      return undefined

    case 'object':
      // For objects, recursively extract defaults from nested properties
      if (property.properties) {
        const nestedDefaults: Record<string, FieldValue> = {}
        let hasDefaults = false

        for (const [key, nestedProp] of Object.entries(property.properties)) {
          if (typeof nestedProp === 'boolean') continue
          const nestedDefault = getDefaultForProperty(nestedProp)
          if (nestedDefault !== undefined) {
            nestedDefaults[key] = nestedDefault
            hasDefaults = true
          }
        }

        return hasDefaults ? nestedDefaults : undefined
      }
      return undefined

    case 'array':
      // Return explicit array default if defined
      if (property.default !== undefined) {
        return property.default as FieldValue
      }
      return undefined

    default:
      return undefined
  }
}

/**
 * Merges user-provided values with schema defaults.
 *
 * User values take precedence, but defaults fill in missing fields.
 *
 * @param values - User-provided form values
 * @param schema - JSON Schema with default values
 * @returns Merged values (user values + defaults)
 */
export function mergeWithDefaults(values: FormValues, schema: JSONSchema7): FormValues {
  const defaults = extractDefaults(schema)
  return {
    ...defaults,
    ...values,
  }
}

/**
 * Checks if a schema has any required fields
 */
export function hasRequiredFields(schema: JSONSchema7): boolean {
  return Boolean(schema.required && schema.required.length > 0)
}

/**
 * Gets list of required field names
 */
export function getRequiredFields(schema: JSONSchema7): string[] {
  return schema.required || []
}

/**
 * Checks if a schema can be auto-submitted (no required fields without EXPLICIT defaults).
 *
 * This only considers explicit defaults ("default": value in schema), not implicit
 * defaults (first enum value, minimum value, etc.). This ensures tools with
 * required enum parameters show a form to the user instead of auto-submitting
 * with an arbitrary first enum value.
 *
 * @param schema - JSON Schema to check
 * @returns true if tool can be auto-submitted without user input
 */
export function canAutoSubmit(schema: JSONSchema7): boolean {
  if (!schema.required || schema.required.length === 0) {
    return true
  }

  if (!schema.properties) return false

  // Check if all required fields have EXPLICIT defaults
  for (const requiredField of schema.required) {
    const property = schema.properties[requiredField]
    if (!property || typeof property === 'boolean' || !hasExplicitDefault(property)) {
      return false  // Required field has no explicit default
    }
  }

  return true
}
