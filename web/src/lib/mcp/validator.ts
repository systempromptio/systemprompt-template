import type { McpOutputSchema, ValidationError } from './types'

interface DataWithItems {
  items: unknown
  [key: string]: unknown
}

function hasItemsProperty(data: unknown): data is DataWithItems {
  return typeof data === 'object' && data !== null && 'items' in data
}

export function validateStructuredContent(
  data: unknown,
  schema: McpOutputSchema
): ValidationError[] {
  const errors: ValidationError[] = []

  if (schema.type === 'object' && (typeof data !== 'object' || data === null || Array.isArray(data))) {
    errors.push({
      path: [],
      message: 'Expected object at root',
      expected: 'object',
      received: Array.isArray(data) ? 'array' : typeof data
    })
    return errors
  }

  if (schema.required && typeof data === 'object' && data !== null) {
    for (const requiredProp of schema.required) {
      if (!(requiredProp in data)) {
        errors.push({
          path: [requiredProp],
          message: `Required property '${requiredProp}' is missing`,
          expected: 'present',
          received: 'missing'
        })
      }
    }
  }

  const artifactType = schema['x-artifact-type']
  if (artifactType === 'table' && typeof data === 'object' && data !== null) {
    if (!hasItemsProperty(data)) {
      errors.push({
        path: ['items'],
        message: 'Table data must have "items" array property',
        expected: 'items property',
        received: 'missing'
      })
    } else if (!Array.isArray(data.items)) {
      errors.push({
        path: ['items'],
        message: 'items must be an array',
        expected: 'array',
        received: typeof data.items
      })
    }
  }

  return errors
}

export function isArrayResponse(data: unknown): data is { items: unknown[] } {
  return (
    hasItemsProperty(data) &&
    Array.isArray(data.items)
  )
}
