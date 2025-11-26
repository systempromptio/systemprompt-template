import type { FormValues } from './types'
import type { JSONSchema7 } from 'json-schema'

/**
 * Populate form fields from cached object by preserving nested structure.
 *
 * The form state structure must match the schema structure for ObjectFields
 * to correctly access nested properties via objectValue[subName].
 *
 * @example
 * { uuid: "...", card: { name: "test" } } → { uuid: "...", card: { name: "test" } }
 *
 * This is a generic utility that works with any object structure,
 * not specific to any particular tool or entity type.
 */
export function populateFormFromObject(
    obj: unknown,
    _schema: JSONSchema7,
    _triggerField: string
): FormValues {
    if (typeof obj !== 'object' || obj === null) return {}

    return obj as FormValues
}
