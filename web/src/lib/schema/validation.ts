import type { JSONSchema7 } from 'json-schema'

/**
 * Validates if a value is a valid JSON Schema Draft 7 object schema
 *
 * @param value - Unknown value to validate
 * @returns True if value is a valid JSON Schema object
 */
export function isJSONSchema7(value: unknown): value is JSONSchema7 {
  if (typeof value !== 'object' || value === null) {
    return false
  }

  const schema = value as Record<string, unknown>

  // Must have type property
  if (!('type' in schema)) {
    return false
  }

  // Type must be 'object' or array including 'object'
  const type = schema.type
  if (type !== 'object' && !(Array.isArray(type) && type.includes('object'))) {
    return false
  }

  // If properties exist, they must be an object
  if ('properties' in schema && typeof schema.properties !== 'object') {
    return false
  }

  // If required exists, it must be an array of strings
  if ('required' in schema) {
    if (!Array.isArray(schema.required)) {
      return false
    }
    if (!schema.required.every((item) => typeof item === 'string')) {
      return false
    }
  }

  return true
}

/**
 * Validates and safely casts an unknown value to JSONSchema7
 *
 * Performs runtime validation to ensure the value is actually a valid
 * JSON Schema before casting. This eliminates unsafe `as unknown as` patterns.
 *
 * @param rawSchema - Unknown value (typically from API/network)
 * @param context - Context for error messages (e.g., tool name)
 * @returns Validated JSON Schema
 * @throws Error if schema is invalid
 */
export function validateAndCastSchema(
  rawSchema: unknown,
  context?: string
): JSONSchema7 {
  const contextPrefix = context ? `[${context}] ` : ''

  // Type guard check
  if (!isJSONSchema7(rawSchema)) {
    const errorMsg = `${contextPrefix}Invalid JSON Schema: Expected object with type='object'`
    throw new SchemaValidationError(errorMsg, rawSchema)
  }

  // Additional validation: check if it's actually usable
  const schema = rawSchema as JSONSchema7

  // Warn if schema has no properties (likely a mistake)
  if (!schema.properties || Object.keys(schema.properties).length === 0) {
  }

  return schema
}

/**
 * Custom error class for schema validation failures
 */
export class SchemaValidationError extends Error {
  readonly invalidSchema: unknown

  constructor(message: string, invalidSchema: unknown) {
    super(message)
    this.name = 'SchemaValidationError'
    this.invalidSchema = invalidSchema
  }
}

/**
 * Safely extracts JSON Schema from MCP tool inputSchema
 *
 * This is a convenience wrapper around validateAndCastSchema
 * specifically for MCP tool schemas.
 *
 * @param inputSchema - The inputSchema from an MCP Tool
 * @param toolName - Tool name for error context
 * @returns Validated JSONSchema7
 */
export function extractToolInputSchema(
  inputSchema: unknown,
  toolName: string
): JSONSchema7 {
  try {
    return validateAndCastSchema(inputSchema, `Tool: ${toolName}`)
  } catch (error) {
    // Provide helpful fallback for tools with no parameters

    // Return a valid empty schema as fallback
    return {
      type: 'object',
      properties: {},
      required: [],
    }
  }
}
