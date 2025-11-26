/**
 * Number input field for JSON Schema forms.
 *
 * Handles integer and decimal numbers with min/max constraints.
 *
 * @module tools/fields/NumberField
 */

import React, { useCallback } from 'react'
import type { JSONSchema7 } from 'json-schema'
import type { FieldValue } from '@/lib/schema/types'
import { formatFieldName } from '../SchemaField'

interface NumberFieldProps {
  name: string
  property: JSONSchema7
  value: FieldValue
  onChange: (value: FieldValue) => void
  error?: string
  required?: boolean
  isInteger: boolean
}

/**
 * Memoized number field component.
 * Renders appropriate input based on isInteger flag.
 */
export const NumberField = React.memo(function NumberField({
  name,
  property,
  value,
  onChange,
  error,
  required,
  isInteger,
}: NumberFieldProps) {
  const numValue = typeof value === 'number' ? value : typeof value === 'string' ? value : ''

  const handleChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      const val = e.target.value
      onChange(val === '' ? '' : isInteger ? parseInt(val) : parseFloat(val))
    },
    [onChange, isInteger]
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
      <input
        type="number"
        id={name}
        value={numValue}
        onChange={handleChange}
        className={`w-full px-3 py-2 border rounded-md bg-surface text-text-primary focus:outline-none focus:ring-2 ${
          error
            ? 'border-red-300 focus:ring-red-500'
            : 'border-border-primary focus:ring-primary'
        }`}
        min={property.minimum}
        max={property.maximum}
        step={isInteger ? 1 : 'any'}
      />
      {error && <p className="text-xs text-red-600 mt-1">{error}</p>}
      {(property.minimum !== undefined || property.maximum !== undefined) && (
        <p className="text-xs text-text-tertiary mt-1">
          {property.minimum !== undefined && property.maximum !== undefined
            ? `Range: ${property.minimum} - ${property.maximum}`
            : property.minimum !== undefined
            ? `Minimum: ${property.minimum}`
            : `Maximum: ${property.maximum}`}
        </p>
      )}
    </div>
  )
})
