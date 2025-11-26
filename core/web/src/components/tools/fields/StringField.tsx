/**
 * String input field for JSON Schema forms.
 *
 * Supports text and textarea variants based on schema maxLength.
 *
 * @module tools/fields/StringField
 */

import React, { useCallback } from 'react'
import type { JSONSchema7 } from 'json-schema'
import type { FieldValue } from '@/lib/schema/types'
import { formatFieldName } from '../SchemaField'

interface StringFieldProps {
  name: string
  property: JSONSchema7
  value: FieldValue
  onChange: (value: FieldValue) => void
  error?: string
  required?: boolean
}

/**
 * Memoized string field component.
 * Automatically switches to textarea when maxLength > 100.
 */
export const StringField = React.memo(function StringField({
  name,
  property,
  value,
  onChange,
  error,
  required,
}: StringFieldProps) {
  const stringValue = typeof value === 'string' ? value : ''
  const isLongText = property.maxLength && property.maxLength > 100

  const handleChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement | HTMLTextAreaElement>) => {
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
      {isLongText ? (
        <textarea
          id={name}
          value={stringValue}
          onChange={handleChange}
          className={`w-full px-3 py-2 border rounded-md bg-surface text-text-primary focus:outline-none focus:ring-2 ${
            error
              ? 'border-red-300 focus:ring-red-500'
              : 'border-border-primary focus:ring-primary'
          }`}
          rows={3}
          placeholder={property.description}
          maxLength={property.maxLength}
        />
      ) : (
        <input
          type="text"
          id={name}
          value={stringValue}
          onChange={handleChange}
          className={`w-full px-3 py-2 border rounded-md bg-surface text-text-primary focus:outline-none focus:ring-2 ${
            error
              ? 'border-red-300 focus:ring-red-500'
              : 'border-border-primary focus:ring-primary'
          }`}
          placeholder={property.description}
          maxLength={property.maxLength}
        />
      )}
      {error && <p className="text-xs text-red-600 mt-1">{error}</p>}
      {property.minLength && property.maxLength && (
        <p className="text-xs text-text-tertiary mt-1">
          {stringValue.length} / {property.maxLength} characters
        </p>
      )}
    </div>
  )
})
