/**
 * Schema field renderer component.
 *
 * Routes to the appropriate field component based on type.
 *
 * @module tools/SchemaFieldRenderer
 */

import React from 'react'
import type { JSONSchema7 } from 'json-schema'
import type { FieldValue } from '@/lib/schema/types'
import { StringField } from './fields/StringField'
import { NumberField } from './fields/NumberField'
import { BooleanField } from './fields/BooleanField'
import { EnumField } from './fields/EnumField'
import { DynamicEnumField } from './fields/DynamicEnumField'
import { ArrayField } from './fields/ArrayField'
import { ObjectField } from './fields/ObjectField'
import { useSchemaFieldRenderer } from './hooks/useSchemaFieldRenderer'

interface SchemaFieldRendererProps {
  name: string
  property: JSONSchema7
  value: FieldValue
  onChange: (value: FieldValue) => void
  error?: string
  errors?: Record<string, string>
  required?: boolean
  onObjectSelect?: (obj: unknown) => void
  onLoadingChange?: (loading: boolean) => void
}

export const SchemaFieldRenderer = React.memo(function SchemaFieldRenderer({
  name,
  property,
  value,
  onChange,
  error,
  errors = {},
  required,
  onObjectSelect,
  onLoadingChange,
}: SchemaFieldRendererProps) {
  const { renderType, dataSource, isInteger } = useSchemaFieldRenderer(property)

  switch (renderType) {
    case 'dynamic-enum':
      return (
        <DynamicEnumField
          name={name}
          property={property}
          value={value}
          onChange={onChange}
          error={error}
          required={required}
          dataSource={dataSource!}
          onObjectSelect={onObjectSelect}
          onLoadingChange={onLoadingChange}
        />
      )

    case 'enum':
      return (
        <EnumField
          name={name}
          property={property}
          value={value}
          onChange={onChange}
          error={error}
          required={required}
        />
      )

    case 'string':
      return (
        <StringField
          name={name}
          property={property}
          value={value}
          onChange={onChange}
          error={error}
          required={required}
        />
      )

    case 'number':
    case 'integer':
      return (
        <NumberField
          name={name}
          property={property}
          value={value}
          onChange={onChange}
          error={error}
          required={required}
          isInteger={isInteger}
        />
      )

    case 'boolean':
      return (
        <BooleanField
          name={name}
          property={property}
          value={value}
          onChange={onChange}
          error={error}
        />
      )

    case 'object':
      return (
        <ObjectField
          name={name}
          property={property}
          value={value}
          onChange={onChange}
          error={error}
          errors={errors}
          required={required}
        />
      )

    case 'array':
      return (
        <ArrayField
          name={name}
          property={property}
          value={value}
          onChange={onChange}
          error={error}
          required={required}
        />
      )

    default:
      return (
        <StringField
          name={name}
          property={property}
          value={value}
          onChange={onChange}
          error={error}
          required={required}
        />
      )
  }
})
