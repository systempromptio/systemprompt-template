/**
 * Hook for determining which field component to render based on schema.
 *
 * @module tools/hooks/useSchemaFieldRenderer
 */

import type { JSONSchema7 } from 'json-schema'
import type { ExtendedJSONSchema7 } from '@/lib/schema/types'
import { getFieldType, isEnumField } from '@/lib/schema/types'

export type FieldType = 'dynamic-enum' | 'enum' | 'string' | 'number' | 'integer' | 'boolean' | 'object' | 'array' | 'default'

export function useSchemaFieldRenderer(property: JSONSchema7) {
  const fieldType = getFieldType(property)
  const hasEnum = isEnumField(property)
  const extendedProperty = property as ExtendedJSONSchema7
  const dataSource = extendedProperty['x-data-source']

  const getRenderType = (): FieldType => {
    if (dataSource) return 'dynamic-enum'
    if (hasEnum) return 'enum'

    switch (fieldType) {
      case 'string':
        return 'string'
      case 'number':
        return 'number'
      case 'integer':
        return 'integer'
      case 'boolean':
        return 'boolean'
      case 'object':
        return 'object'
      case 'array':
        return 'array'
      default:
        return 'default'
    }
  }

  return {
    renderType: getRenderType(),
    dataSource,
    isInteger: fieldType === 'integer',
  }
}
