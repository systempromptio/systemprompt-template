/**
 * Array field for JSON Schema forms.
 *
 * Handles arrays with enum items (checkbox group) or JSON input.
 *
 * @module tools/fields/ArrayField
 */

import React, { useCallback } from 'react'
import type { JSONSchema7 } from 'json-schema'
import type { FieldValue } from '@/lib/schema/types'
import { formatFieldName, formatEnumOption } from '../SchemaField'

interface ArrayFieldProps {
  name: string
  property: JSONSchema7
  value: FieldValue
  onChange: (value: FieldValue) => void
  error?: string
  required?: boolean
}

/**
 * Memoized array field component.
 * Renders checkbox group for enum items or JSON input for other arrays.
 */
export const ArrayField = React.memo(function ArrayField({
  name,
  property,
  value,
  onChange,
  error,
  required,
}: ArrayFieldProps) {
  const arrayValue = Array.isArray(value) ? value : []

  const itemSchema = property.items && typeof property.items === 'object' && !Array.isArray(property.items)
    ? (property.items as JSONSchema7)
    : null

  const hasEnumItems = itemSchema && Array.isArray(itemSchema.enum) && itemSchema.enum.length > 0

  const handleToggle = useCallback(
    (option: string) => {
      if (arrayValue.includes(option)) {
        onChange(arrayValue.filter(item => item !== option))
      } else {
        onChange([...arrayValue, option])
      }
    },
    [arrayValue, onChange]
  )

  const handleJsonChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      try {
        const parsed = JSON.parse(e.target.value)
        if (Array.isArray(parsed)) {
          onChange(parsed)
        }
      } catch {
        // Invalid JSON, ignore
      }
    },
    [onChange]
  )

  if (hasEnumItems && itemSchema) {
    const enumOptions = itemSchema.enum as string[]

    return (
      <div className="mb-4">
        <label className="block text-sm font-medium text-text-primary mb-1">
          {formatFieldName(name)}
          {required && <span className="text-red-500 ml-1">*</span>}
        </label>
        {property.description && (
          <p className="text-xs text-text-secondary mb-2">{property.description}</p>
        )}

        <div className="space-y-2 border border-border-primary rounded-md p-3 bg-surface">
          {enumOptions.map((option) => {
            const stringOption = String(option)
            const isChecked = arrayValue.includes(stringOption)

            return (
              <label key={stringOption} className="flex items-center">
                <input
                  type="checkbox"
                  checked={isChecked}
                  onChange={() => handleToggle(stringOption)}
                  className="w-4 h-4 text-primary border-border-primary rounded focus:ring-primary"
                />
                <span className="ml-2 text-sm text-text-primary">
                  {formatEnumOption(stringOption)}
                </span>
              </label>
            )
          })}
        </div>

        {error && <p className="text-xs text-red-600 mt-1">{error}</p>}

        {arrayValue.length > 0 && (
          <p className="text-xs text-text-secondary mt-1">
            Selected: {arrayValue.join(', ')}
          </p>
        )}
      </div>
    )
  }

  return (
    <div className="mb-4">
      <label htmlFor={name} className="block text-sm font-medium text-text-primary mb-1">
        {formatFieldName(name)}
        {required && <span className="text-red-500 ml-1">*</span>}
      </label>
      {property.description && (
        <p className="text-xs text-text-secondary mb-2">{property.description}</p>
      )}
      <input
        type="text"
        id={name}
        value={Array.isArray(value) ? JSON.stringify(value) : ''}
        onChange={handleJsonChange}
        className={`w-full px-3 py-2 border rounded-md bg-surface text-text-primary focus:outline-none focus:ring-2 ${
          error
            ? 'border-red-300 focus:ring-red-500'
            : 'border-border-primary focus:ring-primary'
        }`}
        placeholder='JSON array (e.g., ["item1", "item2"])'
      />
      {error && <p className="text-xs text-red-600 mt-1">{error}</p>}
      <p className="text-xs text-text-tertiary mt-1">Enter array as JSON</p>
    </div>
  )
})
