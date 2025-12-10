import { useState } from 'react'
import type { FormHints } from '@/types/artifact'
import { FormFieldInput } from './FormFieldInput'
import { SubmissionState } from './SubmissionState'

interface FormRendererProps {
  hints: FormHints
}

export function FormRenderer({ hints }: FormRendererProps) {
  const fields = hints.fields || []
  const [formData, setFormData] = useState<Record<string, unknown>>(() => {
    const initial: Record<string, unknown> = {}
    fields.forEach(field => {
      if (field.default !== undefined) {
        initial[field.name] = field.default
      }
    })
    return initial
  })
  const [errors, setErrors] = useState<Record<string, string>>({})
  const [submitted, setSubmitted] = useState(false)

  const handleChange = (name: string, value: unknown) => {
    setFormData(prev => ({ ...prev, [name]: value }))
    if (errors[name]) {
      setErrors(prev => {
        const newErrors = { ...prev }
        delete newErrors[name]
        return newErrors
      })
    }
  }

  const validate = (): boolean => {
    const newErrors: Record<string, string> = {}

    fields.forEach(field => {
      const value = formData[field.name]

      if (field.required && !value) {
        newErrors[field.name] = `${field.label || field.name} is required`
      }
    })

    setErrors(newErrors)
    return Object.keys(newErrors).length === 0
  }

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()

    if (!validate()) {
      return
    }

    setSubmitted(true)
  }

  const layout = hints.layout || 'vertical'

  if (submitted) {
    return (
      <SubmissionState
        submitAction={hints.submit_action}
        onReset={() => setSubmitted(false)}
      />
    )
  }

  return (
    <form
      onSubmit={handleSubmit}
      className={layout === 'grid' ? 'grid grid-cols-2 gap-4' : 'space-y-4'}
    >
      {fields.map(field => (
        <div key={field.name} className={layout === 'horizontal' ? 'flex items-center gap-4' : ''}>
          <label
            htmlFor={field.name}
            className={`block text-sm font-medium text-primary ${
              layout === 'horizontal' ? 'w-32 flex-shrink-0' : 'mb-1'
            }`}
          >
            {field.label || field.name}
            {field.required && <span className="text-error ml-1">*</span>}
          </label>

          <div className="flex-1">
            <FormFieldInput
              field={field}
              value={formData[field.name]}
              onChange={(value) => handleChange(field.name, value)}
              error={errors[field.name]}
            />

            {field.help_text && !errors[field.name] && (
              <p className="mt-1 text-xs text-secondary">{field.help_text}</p>
            )}

            {errors[field.name] && (
              <p className="mt-1 text-xs text-error">{errors[field.name]}</p>
            )}
          </div>
        </div>
      ))}

      <div className={layout === 'grid' ? 'col-span-2' : ''}>
        <button
          type="submit"
          className="w-full px-4 py-2 bg-primary text-inverted rounded hover:bg-secondary focus:outline-none focus:ring-2 focus:ring-primary"
        >
          Submit
        </button>
      </div>
    </form>
  )
}
