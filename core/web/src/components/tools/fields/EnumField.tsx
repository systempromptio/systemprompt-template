/**
 * Enum dropdown field for JSON Schema forms.
 *
 * Renders static options from schema enum values.
 *
 * @module tools/fields/EnumField
 */

import React, { useCallback } from 'react'
import type { JSONSchema7 } from 'json-schema'
import type { FieldValue } from '@/lib/schema/types'
import { formatFieldName, formatEnumOption } from '../SchemaField'

interface EnumFieldProps {
  name: string
  property: JSONSchema7
  value: FieldValue
  onChange: (value: FieldValue) => void
  error?: string
  required?: boolean
}

/**
 * Memoized enum field component.
 * Renders dropdown with static options from schema.
 */
export const EnumField = React.memo(function EnumField({
  name,
  property,
  value,
  onChange,
  error,
  required,
}: EnumFieldProps) {
  const stringValue = value !== undefined && value !== null ? String(value) : ''

  const handleChange = useCallback(
    (e: React.ChangeEvent<HTMLSelectElement>) => {
      onChange(e.target.value)
    },
    [onChange]
  )

  return (
    <div className="mb-4">
      <label htmlFor={name} className="block text-sm font-medium text-text-primary mb-1">
        {formatFieldName(name)}
        {required && <span className="text-red-500 ml-1">*</span>}
      </label>
      {property.description && (
        <p className="text-xs text-text-secondary mb-2">{property.description}</p>
      )}
      <select
        id={name}
        value={stringValue}
        onChange={handleChange}
        className={`w-full px-3 py-2 border rounded-md bg-surface text-text-primary focus:outline-none focus:ring-2 ${
          error
            ? 'border-red-300 focus:ring-red-500'
            : 'border-border-primary focus:ring-primary'
        }`}
      >
        <option value="">Select...</option>
        {property.enum?.map((option) => (
          <option key={String(option)} value={String(option)}>
            {formatEnumOption(String(option))}
          </option>
        ))}
      </select>
      {error && <p className="text-xs text-red-600 mt-1">{error}</p>}
    </div>
  )
})
