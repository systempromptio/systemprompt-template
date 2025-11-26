import { useMemo, useCallback } from 'react'
import type { Dispatch, SetStateAction } from 'react'
import type { JSONSchema7 } from 'json-schema'
import type { FormValues, FieldValue } from '@/lib/schema/types'
import { isRequired } from '@/lib/schema/types'
import { getDiscriminatorField } from '@/lib/schema/resolver'
import { populateFormFromObject } from '@/lib/schema/formPopulation'
import { SchemaField } from './SchemaField'

interface SchemaFormGeneratorProps {
  schema: JSONSchema7
  values: FormValues
  onChange: Dispatch<SetStateAction<FormValues>>
  errors?: Record<string, string>
  onLoadingChange?: (loading: boolean) => void
}

/**
 * Dynamically generates a form based on a JSON Schema with support for conditional schemas.
 *
 * This component:
 * - Analyzes the schema and creates appropriate form fields
 * - Handles conditional schemas (if/then/else, allOf, oneOf, anyOf)
 * - Dynamically adapts the form when discriminator fields change
 * - Supports all common JSON Schema types
 *
 * @param schema - JSON Schema defining the form structure
 * @param values - Current form values
 * @param onChange - Callback when form values change
 * @param errors - Validation errors to display (field name → error message)
 */
export function SchemaFormGenerator({
  schema,
  values,
  onChange,
  errors = {},
  onLoadingChange,
}: SchemaFormGeneratorProps) {
  // NOTE: Schema is already resolved at parent level (ToolParameterModal)
  // No need to resolve again here

  // Get discriminator field (the field that controls which sub-schema to show)
  const discriminatorField = useMemo(() => {
    return getDiscriminatorField(schema)
  }, [schema])

  const handleFieldChange = (fieldName: string) => (value: FieldValue) => {
    onChange({
      ...values,
      [fieldName]: value,
    })
  }

  // Handle object selection from dynamic dropdowns
  const handleObjectSelect = useCallback(async (obj: unknown, fieldName: string) => {
    // Populate form fields from cached object
    const populated = populateFormFromObject(obj, schema, fieldName)

    // Use function update to avoid stale closure values
    onChange((currentValues) => ({
      ...currentValues,
      ...populated
    }))
  }, [schema, onChange])

  // Sort fields: required first, discriminator field at top, then alphabetical
  const sortedFields = Object.entries(schema.properties || {}).sort(([nameA], [nameB]) => {
    // Discriminator field always first
    if (discriminatorField) {
      if (nameA === discriminatorField) return -1
      if (nameB === discriminatorField) return 1
    }

    const requiredA = isRequired(nameA, schema)
    const requiredB = isRequired(nameB, schema)

    // Required fields first
    if (requiredA && !requiredB) return -1
    if (!requiredA && requiredB) return 1

    // Then alphabetical
    return nameA.localeCompare(nameB)
  })

  return (
    <div className="space-y-2">
      {sortedFields.map(([fieldName, property]) => {
        if (typeof property === 'boolean') return null

        return (
          <SchemaField
            key={fieldName}
            name={fieldName}
            property={property}
            value={values[fieldName]}
            onChange={handleFieldChange(fieldName)}
            onObjectSelect={(obj) => handleObjectSelect(obj, fieldName)}
            onLoadingChange={onLoadingChange}
            error={errors[fieldName]}
            errors={errors}
            required={isRequired(fieldName, schema)}
          />
        )
      })}

      {Object.keys(errors).length > 0 && Object.keys(schema.properties || {}).length === 0 && (
        <div className="p-3 bg-red-50 border border-red-200 rounded-md">
          <p className="text-sm text-red-600">Please correct the errors above</p>
        </div>
      )}
    </div>
  )
}
