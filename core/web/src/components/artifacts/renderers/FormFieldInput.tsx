/**
 * FormFieldInput component.
 *
 * Renders the appropriate input element based on field type.
 *
 * @module artifacts/renderers/FormFieldInput
 */

import React from 'react'
import type { FormField } from '@/types/artifact'

interface FormFieldInputProps {
  field: FormField
  value: unknown
  onChange: (value: unknown) => void
  error?: string
}

export const FormFieldInput = React.memo(function FormFieldInput({
  field,
  value,
  onChange,
  error,
}: FormFieldInputProps) {
  const baseClasses = `w-full px-3 py-2 border rounded bg-surface text-primary focus:outline-none focus:ring-2 focus:ring-primary ${
    error ? 'border-error' : 'border-primary-10'
  }`

  switch (field.type) {
    case 'select':
      return (
        <select
          id={field.name}
          value={String(value || '')}
          onChange={(e) => onChange(e.target.value)}
          className={baseClasses}
        >
          <option value="">Select...</option>
          {field.options?.map((opt) => {
            const optValue = typeof opt === 'string' ? opt : opt.value
            const optLabel = typeof opt === 'string' ? opt : opt.label
            return (
              <option key={optValue} value={optValue}>
                {optLabel}
              </option>
            )
          })}
        </select>
      )

    case 'checkbox':
      return (
        <input
          id={field.name}
          type="checkbox"
          checked={Boolean(value)}
          onChange={(e) => onChange(e.target.checked)}
          className="w-4 h-4 text-primary border-primary-10 rounded focus:ring-primary"
        />
      )

    case 'textarea':
      return (
        <textarea
          id={field.name}
          value={String(value || '')}
          onChange={(e) => onChange(e.target.value)}
          placeholder={field.placeholder}
          rows={4}
          className={baseClasses}
        />
      )

    case 'number':
      return (
        <input
          id={field.name}
          type="number"
          value={String(value || '')}
          onChange={(e) => onChange(e.target.value ? Number(e.target.value) : '')}
          placeholder={field.placeholder}
          className={baseClasses}
        />
      )

    case 'email':
    case 'password':
    case 'date':
    case 'datetime':
    case 'text':
    default:
      return (
        <input
          id={field.name}
          type={field.type === 'datetime' ? 'datetime-local' : field.type}
          value={String(value || '')}
          onChange={(e) => onChange(e.target.value)}
          placeholder={field.placeholder}
          className={baseClasses}
        />
      )
  }
})
