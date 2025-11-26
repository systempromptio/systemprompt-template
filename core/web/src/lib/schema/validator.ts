import type { JSONSchema7 } from 'json-schema'
import type {
  FormValues,
  ValidationResult,
  FieldValue,
} from './types'
import { getFieldType, isRequired } from './types'

/**
 * Validates form values against a JSON Schema.
 *
 * Performs comprehensive validation including:
 * - Required field checks
 * - Type validation and coercion
 * - Min/max constraints
 * - Enum validation
 * - String length, pattern, and format validation (email, uri, uuid, date-time, etc.)
 *
 * @param values - Form values to validate
 * @param schema - JSON Schema defining the structure and constraints
 * @returns ValidationResult with valid flag and error messages
 */
export function validateAgainstSchema(
  values: FormValues,
  schema: JSONSchema7
): ValidationResult {
  const errors: Record<string, string> = {}

  // Check required fields
  if (schema.required) {
    for (const fieldName of schema.required) {
      const value = values[fieldName]
      if (value === undefined || value === null || value === '') {
        errors[fieldName] = 'This field is required'
      }
    }
  }

  // Validate each field
  if (schema.properties) {
    for (const [fieldName, property] of Object.entries(schema.properties)) {
      // Skip boolean schemas
      if (typeof property === 'boolean') continue

      const value = values[fieldName]

      // Skip optional fields that are not provided
      if (!isRequired(fieldName, schema) && (value === undefined || value === null || value === '')) {
        continue
      }

      // Skip if already has required error
      if (errors[fieldName]) {
        continue
      }

      const fieldType = getFieldType(property)

      // Handle object type specially for nested validation
      if (fieldType === 'object') {
        validateObject(value, property, fieldName, schema, errors)
      } else {
        const fieldError = validateField(fieldName, value, property)
        if (fieldError) {
          errors[fieldName] = fieldError
        }
      }
    }
  }

  return {
    valid: Object.keys(errors).length === 0,
    errors,
  }
}

/**
 * Validates a single field value against its schema property
 */
function validateField(
  _fieldName: string,
  value: FieldValue,
  property: JSONSchema7
): string | null {
  const fieldType = getFieldType(property)

  // Enum validation (highest priority)
  if (property.enum && Array.isArray(property.enum)) {
    // Check if value matches any enum value (with type coercion for strings/numbers)
    const matchesEnum = property.enum.some(enumVal => {
      if (enumVal === value) return true
      // Allow string representation comparison for numbers
      if (String(enumVal) === String(value)) return true
      return false
    })

    if (!matchesEnum) {
      return `Must be one of: ${property.enum.join(', ')}`
    }
  }

  // Type-specific validation
  switch (fieldType) {
    case 'string':
      return validateString(value, property)
    case 'number':
    case 'integer':
      return validateNumber(value, property, fieldType)
    case 'boolean':
      return validateBoolean(value)
    case 'object':
      // Object validation is handled differently (returns void, mutates errors object)
      // This case should not be reached for top-level validation
      return null
    case 'array':
      return validateArray(value, property)
    default:
      return null
  }
}

/**
 * Validate string format (email, uri, uuid, date-time, etc.)
 */
function validateFormat(value: string, format: string): string | null {
  switch (format) {
    case 'email':
      const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/
      return emailRegex.test(value) ? null : 'Invalid email address'

    case 'uri':
    case 'url':
      try {
        new URL(value)
        return null
      } catch {
        return 'Invalid URL'
      }

    case 'uuid':
      const uuidRegex = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i
      return uuidRegex.test(value) ? null : 'Invalid UUID'

    case 'date':
      const dateRegex = /^\d{4}-\d{2}-\d{2}$/
      return dateRegex.test(value) && !isNaN(Date.parse(value)) ? null : 'Invalid date (use YYYY-MM-DD)'

    case 'date-time':
      return !isNaN(Date.parse(value)) ? null : 'Invalid date-time'

    case 'time':
      const timeRegex = /^\d{2}:\d{2}(:\d{2})?$/
      return timeRegex.test(value) ? null : 'Invalid time (use HH:MM or HH:MM:SS)'

    case 'ipv4':
      const ipv4Regex = /^(\d{1,3}\.){3}\d{1,3}$/
      return ipv4Regex.test(value) ? null : 'Invalid IPv4 address'

    case 'ipv6':
      const ipv6Regex = /^([0-9a-f]{0,4}:){7}[0-9a-f]{0,4}$/i
      return ipv6Regex.test(value) ? null : 'Invalid IPv6 address'

    case 'hostname':
      const hostnameRegex = /^[a-z0-9]([a-z0-9-]{0,61}[a-z0-9])?(\.[a-z0-9]([a-z0-9-]{0,61}[a-z0-9])?)*$/i
      return hostnameRegex.test(value) ? null : 'Invalid hostname'

    default:
      return null // Unknown format, pass validation
  }
}

/**
 * Validate string field
 */
function validateString(value: FieldValue, property: JSONSchema7): string | null {
  if (typeof value !== 'string') {
    return 'Must be a string'
  }

  if (property.minLength !== undefined && value.length < property.minLength) {
    return `Must be at least ${property.minLength} characters`
  }

  if (property.maxLength !== undefined && value.length > property.maxLength) {
    return `Must be at most ${property.maxLength} characters`
  }

  if (property.format) {
    const formatError = validateFormat(value, property.format)
    if (formatError) {
      return formatError
    }
  }

  if (property.pattern) {
    try {
      const regex = new RegExp(property.pattern)
      if (!regex.test(value)) {
        return 'Invalid format'
      }
    } catch (e) {
      // Invalid regex pattern, skip validation
    }
  }

  return null
}

/**
 * Validate number/integer field
 */
function validateNumber(
  value: FieldValue,
  property: JSONSchema7,
  type: 'number' | 'integer'
): string | null {
  // Try to coerce string to number
  let numValue: number
  if (typeof value === 'string') {
    numValue = parseFloat(value)
  } else if (typeof value === 'number') {
    numValue = value
  } else {
    return 'Must be a number'
  }

  if (isNaN(numValue)) {
    return 'Must be a valid number'
  }

  // Integer check
  if (type === 'integer' && !Number.isInteger(numValue)) {
    return 'Must be an integer'
  }

  // Min/max validation
  if (property.minimum !== undefined && numValue < property.minimum) {
    return `Must be at least ${property.minimum}`
  }

  if (property.maximum !== undefined && numValue > property.maximum) {
    return `Must be at most ${property.maximum}`
  }

  return null
}

/**
 * Validate boolean field
 */
function validateBoolean(value: FieldValue): string | null {
  if (typeof value !== 'boolean') {
    return 'Must be true or false'
  }
  return null
}

/**
 * Validate object field and its nested properties.
 * Returns null if valid, or recursively validates nested properties.
 */
function validateObject(value: FieldValue, property: JSONSchema7, fieldName: string, _schema: JSONSchema7, errors: Record<string, string>): void {
  if (typeof value !== 'object' || value === null || Array.isArray(value)) {
    errors[fieldName] = 'Must be an object'
    return
  }

  // If the property has nested properties, validate them
  if (property.properties) {
    for (const [nestedName, nestedProp] of Object.entries(property.properties)) {
      if (typeof nestedProp === 'boolean') continue

      const nestedValue = (value as Record<string, FieldValue>)[nestedName]
      const nestedPath = `${fieldName}.${nestedName}`

      // Check if nested field is required
      const nestedRequired = Array.isArray(property.required) && property.required.includes(nestedName)

      // Skip optional fields that are not provided
      if (!nestedRequired && (nestedValue === undefined || nestedValue === null || nestedValue === '')) {
        continue
      }

      // Required field validation
      if (nestedRequired && (nestedValue === undefined || nestedValue === null || nestedValue === '')) {
        errors[nestedPath] = 'This field is required'
        continue
      }

      // Validate the nested field
      const nestedError = validateField(nestedPath, nestedValue, nestedProp)
      if (nestedError) {
        errors[nestedPath] = nestedError
      }
    }
  }
}

/**
 * Validate array field
 */
function validateArray(value: FieldValue, property?: JSONSchema7): string | null {
  if (!Array.isArray(value)) {
    return 'Must be an array'
  }

  if (!property) return null

  if (property.minItems !== undefined && value.length < property.minItems) {
    return `Must have at least ${property.minItems} items`
  }

  if (property.maxItems !== undefined && value.length > property.maxItems) {
    return `Must have at most ${property.maxItems} items`
  }

  if (property.items && typeof property.items === 'object' && !Array.isArray(property.items)) {
    const itemSchema = property.items as JSONSchema7

    if (itemSchema.enum && Array.isArray(itemSchema.enum)) {
      for (const item of value) {
        const matchesEnum = itemSchema.enum.some(enumVal =>
          enumVal === item || String(enumVal) === String(item)
        )
        if (!matchesEnum) {
          return `All items must be one of: ${itemSchema.enum.join(', ')}`
        }
      }
    }
  }

  return null
}

/**
 * Coerce form values to match schema types.
 *
 * This is useful for converting string inputs to numbers, etc.
 *
 * IMPORTANT: Preserves all input values, only coercing types for fields in schema.
 * This ensures conditional schema fields don't get dropped.
 *
 * @param values - Raw form values
 * @param schema - JSON Schema
 * @returns Coerced values matching schema types
 */
export function coerceValues(values: FormValues, schema: JSONSchema7): FormValues {
  // Start with all values to preserve fields not in current schema
  const coerced: FormValues = { ...values }

  if (!schema.properties) return coerced

  for (const [fieldName, property] of Object.entries(schema.properties)) {
    // Skip boolean schemas
    if (typeof property === 'boolean') continue

    const value = values[fieldName]

    if (value === undefined || value === null || value === '') {
      // Keep undefined/null/empty as-is
      continue
    }

    const fieldType = getFieldType(property)

    switch (fieldType) {
      case 'number':
      case 'integer':
        // Coerce string to number
        if (typeof value === 'string') {
          const numValue = parseFloat(value)
          coerced[fieldName] = isNaN(numValue) ? value : numValue
        } else {
          coerced[fieldName] = value
        }
        break

      case 'boolean':
        // Coerce string to boolean
        if (typeof value === 'string') {
          coerced[fieldName] = value === 'true' || value === '1'
        } else {
          coerced[fieldName] = value
        }
        break

      default:
        coerced[fieldName] = value
    }
  }

  return coerced
}

/**
 * Extract only fields defined in schema properties.
 *
 * This filters form values to only include fields that exist in the schema,
 * removing auto-populated display fields that shouldn't be submitted.
 *
 * @param values - Form values (may include extra fields)
 * @param schema - JSON Schema defining expected fields
 * @returns Filtered values containing only schema fields
 *
 * @example
 * ```ts
 * const allValues = { action: 'delete', uuid: '123', 'card.name': 'test' }
 * const schema = { properties: { action: {...}, uuid: {...} } }
 * const clean = extractSchemaFields(allValues, schema)
 * // Returns: { action: 'delete', uuid: '123' }
 * ```
 */
export function extractSchemaFields(
  values: FormValues,
  schema: JSONSchema7
): FormValues {
  const result: FormValues = {}

  if (!schema.properties) return result

  for (const fieldName of Object.keys(schema.properties)) {
    if (values[fieldName] !== undefined) {
      result[fieldName] = values[fieldName]
    }
  }

  return result
}
