import type { JSONSchema7 } from 'json-schema'
import type { FormValues } from './types'

/**
 * Conditional JSON Schema Resolver
 *
 * Transforms conditional schemas (if/then/else, allOf, oneOf, anyOf) into
 * concrete schemas based on form field values. Enables adaptive forms that
 * show/hide fields based on user selections.
 *
 * Resolution Pipeline:
 * 1. Extract discriminator properties (root control fields)
 * 2. Resolve allOf conditionals (merge matching branches)
 * 3. Resolve if/then/else (single conditional)
 * 4. Resolve oneOf (first match)
 * 5. Resolve anyOf (all matches merged)
 *
 * @example
 * ```ts
 * const schema = {
 *   type: 'object',
 *   properties: { accountType: { enum: ['personal', 'business'] } },
 *   allOf: [{
 *     if: { properties: { accountType: { const: 'business' } } },
 *     then: { properties: { companyName: { type: 'string' } } }
 *   }]
 * }
 * const resolved = resolveConditionalSchema(schema, { accountType: 'business' })
 * ```
 */

/**
 * Resolves conditional JSON Schema for given form values
 *
 * Evaluates all conditionals (if/then/else, allOf, oneOf, anyOf) and returns
 * a schema containing only properties applicable to provided field values.
 *
 * @param schema - JSON Schema with conditional keywords
 * @param values - Current form field values to evaluate conditions against
 * @returns Resolved schema with fields specific to provided values
 *
 * @example
 * ```ts
 * const schema = {
 *   properties: { shippingMethod: { enum: ['delivery', 'pickup'] } },
 *   allOf: [{
 *     if: { properties: { shippingMethod: { const: 'delivery' } } },
 *     then: { properties: { address: { type: 'string' } } }
 *   }]
 * }
 * const resolved = resolveConditionalSchema(schema, { shippingMethod: 'delivery' })
 * ```
 */
export function resolveConditionalSchema(
  schema: JSONSchema7,
  values: FormValues
): JSONSchema7 {
  const discriminatorProps = extractDiscriminatorProperties(schema)
  let resolved: JSONSchema7 = {
    type: 'object',
    properties: discriminatorProps,
    required: schema.required || []
  }

  if (schema.allOf && Array.isArray(schema.allOf)) {
    resolved = resolveAllOf(schema.allOf, values, resolved)
  }

  if (schema.if) {
    resolved = resolveIfThenElse(schema, values, resolved)
  }

  if (schema.oneOf && Array.isArray(schema.oneOf)) {
    resolved = resolveOneOf(schema.oneOf, values, resolved)
  }

  if (schema.anyOf && Array.isArray(schema.anyOf)) {
    resolved = resolveAnyOf(schema.anyOf, values, resolved)
  }

  return resolved
}

/**
 * Identifies discriminator properties controlling conditional branches
 *
 * In conditional schemas, discriminators are required root-level fields whose
 * values determine which conditional branches apply.
 *
 * @param schema - JSON Schema to extract discriminators from
 * @returns Discriminator properties (field name → schema)
 *
 * @example
 * ```ts
 * const schema = {
 *   properties: { method: { enum: ['api', 'webhook'] } },
 *   required: ['method']
 * }
 * const discriminators = extractDiscriminatorProperties(schema)
 * ```
 */
function extractDiscriminatorProperties(schema: JSONSchema7): Record<string, JSONSchema7> {
  if (!schema.properties || typeof schema.properties !== 'object') {
    return {}
  }

  const hasConditionals = schema.allOf || schema.oneOf || schema.anyOf || schema.if
  if (!hasConditionals) {
    const result: Record<string, JSONSchema7> = {}
    for (const [key, value] of Object.entries(schema.properties)) {
      if (typeof value === 'object') {
        result[key] = value
      }
    }
    return result
  }

  if (!schema.required || !Array.isArray(schema.required)) {
    return {}
  }

  const result: Record<string, JSONSchema7> = {}
  for (const fieldName of schema.required) {
    const prop = schema.properties[fieldName]
    if (prop && typeof prop === 'object') {
      result[fieldName] = prop
    }
  }
  return result
}

/**
 * Resolves allOf conditionals by evaluating and merging matching schemas
 *
 * For each schema in allOf:
 * - If has if/then/else: evaluate and merge result
 * - If has properties: merge unconditionally
 * - Skip boolean schemas
 *
 * @param allOf - Schemas to evaluate
 * @param values - Form values for condition matching
 * @param base - Base schema to merge results into
 * @returns Schema with allOf conditionals resolved
 */
function resolveAllOf(
  allOf: (JSONSchema7 | boolean)[],
  values: FormValues,
  base: JSONSchema7
): JSONSchema7 {
  let resolved = base

  for (const subSchema of allOf) {
    if (typeof subSchema !== 'object') {
      continue
    }

    if (subSchema.if) {
      const matched = resolveIfThenElse(subSchema, values, { type: 'object', properties: {}, required: [] })
      if (matched.properties && Object.keys(matched.properties).length > 0) {
        resolved = mergeSchemas(resolved, matched)
      }
    } else if (subSchema.properties) {
      resolved = mergeSchemas(resolved, subSchema)
    }
  }

  return resolved
}

/**
 * Resolves if/then/else by evaluating condition and applying matching branch
 *
 * Returns then branch if condition matches, else branch if not, or base schema.
 *
 * @param schema - Schema with if/then/else keywords
 * @param values - Form values to evaluate condition against
 * @param base - Schema when no branch applies
 * @returns Schema from matching branch or base
 *
 * @example
 * ```ts
 * const schema = {
 *   if: { properties: { isCompany: { const: true } } },
 *   then: { properties: { taxId: { type: 'string' } } },
 *   else: { properties: { ssn: { type: 'string' } } }
 * }
 * const resolved = resolveIfThenElse(schema, { isCompany: true }, {})
 * ```
 */
function resolveIfThenElse(
  schema: JSONSchema7,
  values: FormValues,
  base: JSONSchema7
): JSONSchema7 {
  if (!schema.if || typeof schema.if !== 'object') {
    return base
  }

  const matches = matchesCondition(schema.if, values)

  if (matches && schema.then && typeof schema.then === 'object') {
    return mergeSchemas(base, schema.then)
  }

  if (!matches && schema.else && typeof schema.else === 'object') {
    return mergeSchemas(base, schema.else)
  }

  return base
}

/**
 * Resolves oneOf by returning first matching schema
 *
 * Evaluates each schema in order and merges first match into base.
 * Unlike anyOf, stops after first match (one-of semantics).
 *
 * @param oneOf - Schemas to evaluate
 * @param values - Form values to evaluate conditions against
 * @param base - Base schema to merge first match into
 * @returns Schema with first matching oneOf option
 */
function resolveOneOf(
  oneOf: (JSONSchema7 | boolean)[],
  values: FormValues,
  base: JSONSchema7
): JSONSchema7 {
  for (const subSchema of oneOf) {
    if (typeof subSchema === 'object' && matchesCondition(subSchema, values)) {
      return mergeSchemas(base, subSchema)
    }
  }

  return base
}

/**
 * Resolves anyOf by merging all matching schemas
 *
 * Evaluates each schema and merges results for all matches.
 * Unlike oneOf, all matches are merged (any-of semantics).
 *
 * @param anyOf - Schemas to evaluate
 * @param values - Form values to evaluate conditions against
 * @param base - Base schema to merge matches into
 * @returns Schema with all matching anyOf options merged
 */
function resolveAnyOf(
  anyOf: (JSONSchema7 | boolean)[],
  values: FormValues,
  base: JSONSchema7
): JSONSchema7 {
  let resolved = base

  for (const subSchema of anyOf) {
    if (typeof subSchema === 'object' && matchesCondition(subSchema, values)) {
      resolved = mergeSchemas(resolved, subSchema)
    }
  }

  return resolved
}

/**
 * Evaluates if form values satisfy all constraints in a condition
 *
 * Checks property constraints:
 * - const: value must equal specified constant
 * - enum: value must be in allowed values (supports type coercion)
 * - required: value must be non-empty if in required array
 *
 * All constraints must pass for condition to match.
 *
 * @param condition - Condition schema with constraints
 * @param values - Form values to evaluate
 * @returns True if all constraints are satisfied
 *
 * @example
 * ```ts
 * const condition = {
 *   properties: {
 *     role: { const: 'admin' },
 *     region: { enum: ['US', 'EU'] }
 *   },
 *   required: ['role']
 * }
 * const matches = matchesCondition(condition, { role: 'admin', region: 'US' })
 * ```
 */
function matchesCondition(
  condition: JSONSchema7,
  values: FormValues
): boolean {
  if (!condition.properties) {
    return true
  }

  for (const [key, propSchema] of Object.entries(condition.properties)) {
    if (typeof propSchema !== 'object') {
      continue
    }

    const value = values[key]

    if ('const' in propSchema && propSchema.const !== undefined) {
      if (value !== propSchema.const) {
        return false
      }
    }

    if (propSchema.enum && Array.isArray(propSchema.enum)) {
      const found = propSchema.enum.some(enumVal =>
        enumVal === value || String(enumVal) === String(value)
      )
      if (!found) {
        return false
      }
    }

    if (Array.isArray(condition.required) && condition.required.includes(key)) {
      const hasValue = value !== undefined && value !== null && value !== ''
      if (!hasValue) {
        return false
      }
    }
  }

  return true
}

/**
 * Merges two schemas without overwriting existing properties
 *
 * Combines properties and required arrays. New properties are added only if
 * they don't exist in base, preserving base constraints like enums.
 *
 * @param base - Base schema (takes precedence for existing properties)
 * @param additional - Schema with properties to merge in
 * @returns Merged schema
 *
 * @example
 * ```ts
 * const base = { properties: { firstName: { type: 'string' } } }
 * const additional = { properties: { lastName: { type: 'string' } } }
 * const merged = mergeSchemas(base, additional)
 * ```
 */
function mergeSchemas(base: JSONSchema7, additional: JSONSchema7): JSONSchema7 {
  const baseProps = (base.properties || {}) as Record<string, JSONSchema7>
  const additionalProps = (additional.properties || {}) as Record<string, JSONSchema7>

  const mergedProperties: Record<string, JSONSchema7> = { ...baseProps }
  for (const [key, value] of Object.entries(additionalProps)) {
    if (typeof value === 'object' && !(key in baseProps)) {
      mergedProperties[key] = value
    }
  }

  const merged: JSONSchema7 = {
    type: 'object',
    properties: mergedProperties,
    required: [...new Set([...(base.required || []), ...(additional.required || [])])]
  }

  if (additional.additionalProperties !== undefined) {
    merged.additionalProperties = additional.additionalProperties
  } else if (base.additionalProperties !== undefined) {
    merged.additionalProperties = base.additionalProperties
  }

  return merged
}

/**
 * Identifies the primary discriminator field in a conditional schema
 *
 * Searches for the first field controlling conditional branches in this order:
 * 1. allOf if/then conditionals
 * 2. Root if/then conditional
 * 3. oneOf options
 *
 * @param schema - Conditional schema to inspect
 * @returns Discriminator field name or null if none found
 *
 * @example
 * ```ts
 * const schema = {
 *   properties: { userType: { enum: ['admin', 'user'] } },
 *   allOf: [{
 *     if: { properties: { userType: { const: 'admin' } } },
 *     then: { properties: { permissions: { ... } } }
 *   }]
 * }
 * const field = getDiscriminatorField(schema)
 * ```
 */
export function getDiscriminatorField(schema: JSONSchema7): string | null {
  if (schema.allOf && Array.isArray(schema.allOf)) {
    for (const subSchema of schema.allOf) {
      if (typeof subSchema === 'object' && subSchema.if && typeof subSchema.if === 'object' && subSchema.if.properties) {
        const keys = Object.keys(subSchema.if.properties)
        if (keys.length > 0) {
          return keys[0]
        }
      }
    }
  }

  if (schema.if && typeof schema.if === 'object' && schema.if.properties) {
    const keys = Object.keys(schema.if.properties)
    if (keys.length > 0) {
      return keys[0]
    }
  }

  if (schema.oneOf && Array.isArray(schema.oneOf)) {
    for (const subSchema of schema.oneOf) {
      if (typeof subSchema === 'object' && subSchema.properties) {
        const keys = Object.keys(subSchema.properties)
        if (keys.length > 0) {
          return keys[0]
        }
      }
    }
  }

  return null
}
