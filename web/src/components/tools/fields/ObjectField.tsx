/**
 * Object field for JSON Schema forms.
 *
 * Renders nested object with recursive sub-fields.
 *
 * @module tools/fields/ObjectField
 */

import React, { useCallback } from 'react'
import type { JSONSchema7 } from 'json-schema'
import type { FieldValue } from '@/lib/schema/types'
import { SchemaField } from '../SchemaField'
import { formatFieldName } from '../SchemaField'

interface ObjectFieldProps {
  name: string
  property: JSONSchema7
  value: FieldValue
  onChange: (value: FieldValue) => void
  error?: string
  errors?: Record<string, string>
  required?: boolean
}

/**
 * Memoized object field component.
 * Renders nested object with recursive SchemaField components.
 */
export const ObjectField = React.memo(function ObjectField({
  name,
  property,
  value,
  onChange,
  error,
  errors = {},
  required,
}: ObjectFieldProps) {
  const objectValue = typeof value === 'object' && value !== null && !Array.isArray(value)
    ? (value as Record<string, FieldValue>)
    : {}

  const handleSubFieldChange = useCallback(
    (subFieldName: string) => (subValue: FieldValue) => {
      onChange({
        ...objectValue,
        [subFieldName]: subValue,
      })
    },
    [objectValue, onChange]
  )

  const getSubFieldError = useCallback(
    (subFieldName: string): string | undefined => {
      const nestedPath = `${name}.${subFieldName}`
      return errors[nestedPath]
    },
    [name, errors]
  )

  const subProperties = property.properties || {}
  const subRequired = property.required || []

  return (
    <fieldset className="mb-4 border border-border-secondary rounded-lg p-4 bg-surface-variant">
      <legend className="text-sm font-medium text-text-primary px-2">
        {formatFieldName(name)}
        {required && <span className="text-red-500 ml-1">*</span>}
      </legend>
      {property.description && (
        <p className="text-xs text-text-secondary mb-3">{property.description}</p>
      )}

      <div className="space-y-2">
        {Object.entries(subProperties).map(([subName, subProp]) => {
          if (typeof subProp === 'boolean') return null

          return (
            <SchemaField
              key={subName}
              name={subName}
              property={subProp}
              value={objectValue[subName]}
              onChange={handleSubFieldChange(subName)}
              error={getSubFieldError(subName)}
              errors={errors}
              required={subRequired.includes(subName)}
            />
          )
        })}
      </div>

      {error && <p className="text-xs text-red-600 mt-2">{error}</p>}
    </fieldset>
  )
})
