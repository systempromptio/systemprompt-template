/**
 * Boolean checkbox field for JSON Schema forms.
 *
 * Renders a single checkbox for boolean values.
 *
 * @module tools/fields/BooleanField
 */

import React, { useCallback } from 'react'
import type { JSONSchema7 } from 'json-schema'
import type { FieldValue } from '@/lib/schema/types'
import { formatFieldName } from '../SchemaField'

interface BooleanFieldProps {
  name: string
  property: JSONSchema7
  value: FieldValue
  onChange: (value: FieldValue) => void
  error?: string
}

/**
 * Memoized boolean field component.
 */
export const BooleanField = React.memo(function BooleanField({
  name,
  property,
  value,
  onChange,
  error,
}: BooleanFieldProps) {
  const boolValue = Boolean(value)

  const handleChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      onChange(e.target.checked)
    },
    [onChange]
  )

  return (
    <div className="mb-4">
      <label className="flex items-center">
        <input
          type="checkbox"
          checked={boolValue}
          onChange={handleChange}
          className="w-4 h-4 text-primary border-border-primary rounded focus:ring-primary"
        />
        <span className="ml-2 text-sm font-medium text-text-primary">
          {formatFieldName(name)}
        </span>
      </label>
      {property.description && (
        <p className="text-xs text-text-secondary mt-1 ml-6">{property.description}</p>
      )}
      {error && <p className="text-xs text-red-600 mt-1 ml-6">{error}</p>}
    </div>
  )
})
