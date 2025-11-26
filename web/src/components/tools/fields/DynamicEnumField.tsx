/**
 * Dynamic enum dropdown field for JSON Schema forms.
 *
 * Fetches options from a data source instead of static schema.
 *
 * @module tools/fields/DynamicEnumField
 */

import React, { useCallback } from 'react'
import type { JSONSchema7 } from 'json-schema'
import type { FieldValue, DataSourceConfig } from '@/lib/schema/types'
import { useDynamicOptions } from '@/hooks/useDynamicOptions'
import { formatFieldName } from '../SchemaField'

interface DynamicEnumFieldProps {
  name: string
  property: JSONSchema7
  value: FieldValue
  onChange: (value: FieldValue) => void
  error?: string
  required?: boolean
  dataSource: DataSourceConfig
  onObjectSelect?: (obj: unknown) => void
  onLoadingChange?: (loading: boolean) => void
}

/**
 * Memoized dynamic enum field component.
 * Fetches options asynchronously from a data source.
 */
export const DynamicEnumField = React.memo(function DynamicEnumField({
  name,
  property,
  value,
  onChange,
  error,
  required,
  dataSource,
  onObjectSelect,
  onLoadingChange,
}: DynamicEnumFieldProps) {
  const stringValue = value !== undefined && value !== null ? String(value) : ''
  const { options, loading, error: fetchError, fetchFullObject } = useDynamicOptions(dataSource)

  const handleChange = useCallback(
    async (e: React.ChangeEvent<HTMLSelectElement>) => {
      const selectedValue = e.target.value
      onChange(selectedValue)

      if (onObjectSelect && selectedValue && dataSource.cache_object) {
        onLoadingChange?.(true)
        try {
          const fullObject = await fetchFullObject(selectedValue)
          if (fullObject) {
            onObjectSelect(fullObject)
          }
        } catch (err) {
          const selectedOption = options.find(opt => opt.value === selectedValue)
          if (selectedOption?.data) {
            onObjectSelect(selectedOption.data)
          }
        } finally {
          onLoadingChange?.(false)
        }
      }
    },
    [onChange, onObjectSelect, onLoadingChange, dataSource.cache_object, fetchFullObject, options]
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

      {loading ? (
        <div className="w-full px-3 py-2 border border-border-primary rounded-md bg-surface-variant">
          <p className="text-sm text-text-secondary">Loading options...</p>
        </div>
      ) : fetchError ? (
        <div className="space-y-2">
          <div className="w-full px-3 py-2 border border-red-200 rounded-md bg-red-50">
            <p className="text-sm text-red-600">Failed to load options: {fetchError}</p>
          </div>
          <input
            type="text"
            id={name}
            value={stringValue}
            onChange={(e) => onChange(e.target.value)}
            className="w-full px-3 py-2 border border-border-primary rounded-md bg-surface text-text-primary focus:outline-none focus:ring-2 focus:ring-primary"
            placeholder="Enter value manually"
          />
        </div>
      ) : (
        <select
          id={name}
          value={stringValue}
          onChange={handleChange}
          className={`w-full px-3 py-2 border rounded-md bg-surface text-text-primary focus:outline-none focus:ring-2 ${
            error
              ? 'border-red-300 focus:ring-red-500'
              : 'border-border-primary focus:ring-primary'
          }`}
          disabled={options.length === 0}
        >
          <option value="">Select...</option>
          {options.map((option) => (
            <option key={option.value} value={option.value}>
              {option.label}
            </option>
          ))}
        </select>
      )}

      {error && <p className="text-xs text-red-600 mt-1">{error}</p>}

      {!loading && !fetchError && options.length === 0 && (
        <p className="text-xs text-text-secondary mt-1">No options available</p>
      )}
    </div>
  )
})
