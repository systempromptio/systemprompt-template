import { useState, useEffect } from 'react'
import { Play, AlertTriangle } from 'lucide-react'
import type { McpTool } from '@/stores/tools.store'
import type { FormValues } from '@/lib/schema/types'
import { extractDefaults } from '@/lib/schema/defaults'
import { validateAgainstSchema, coerceValues, extractSchemaFields } from '@/lib/schema/validator'
import { extractToolInputSchema } from '@/lib/schema/validation'
import { useResolvedSchema } from '@/hooks/useResolvedSchema'
import { SchemaFormGenerator } from './SchemaFormGenerator'
import { Modal, ModalBody, ModalFooter } from '@/components/ui/Modal'
import { Button } from '@/components/ui/Button'
import { Icon } from '@/components/ui/Icon'

interface ToolParameterModalProps {
  isOpen: boolean
  tool: McpTool | null
  onClose: () => void
  onSubmit: (tool: McpTool, parameters: FormValues) => Promise<void>
}

export function ToolParameterModal({
  isOpen,
  tool,
  onClose,
  onSubmit,
}: ToolParameterModalProps) {
  const [values, setValues] = useState<FormValues>({})
  const [errors, setErrors] = useState<Record<string, string>>({})
  const [isSubmitting, setIsSubmitting] = useState(false)
  const [isLoadingData, setIsLoadingData] = useState(false)
  const [schemaError, setSchemaError] = useState<string | null>(null)

  useEffect(() => {
    if (!isOpen || !tool) {
      setValues({})
      setErrors({})
      setIsSubmitting(false)
      setSchemaError(null)
      return
    }

    try {
      const schema = extractToolInputSchema(tool.inputSchema, tool.name)
      const defaults = extractDefaults(schema)
      setValues(defaults)
      setErrors({})
      setSchemaError(null)
    } catch (error) {
      const errorMsg = error instanceof Error ? error.message : 'Invalid tool schema'
      setSchemaError(errorMsg)
      setValues({})
    }
  }, [isOpen, tool])

  const baseSchema = tool ? extractToolInputSchema(tool.inputSchema, tool.name) : { type: 'object' as const, properties: {} }
  const resolvedSchema = useResolvedSchema(baseSchema, values)

  if (!isOpen || !tool) {
    return null
  }

  if (schemaError) {
    return (
      <Modal
        isOpen={isOpen}
        onClose={onClose}
        variant="error"
        size="md"
      >
        <ModalBody>
          <div className="flex items-start gap-sm">
            <Icon icon={AlertTriangle} size="lg" color="error" className="flex-shrink-0 mt-0.5" />
            <div className="flex-1">
              <h3 className="text-lg font-heading font-semibold text-text-primary mb-sm">Invalid Tool Schema</h3>
              <p className="text-sm text-text-secondary mb-md">
                The tool's parameter schema is invalid and cannot be displayed.
              </p>
              <div className="p-sm bg-error/10 border border-error/30 rounded-md mb-md">
                <p className="text-sm text-error font-mono">{schemaError}</p>
              </div>
              <p className="text-xs text-text-secondary">
                This is likely an issue with the MCP server implementation.
                Please contact the server administrator.
              </p>
            </div>
          </div>
        </ModalBody>
        <ModalFooter>
          <Button variant="secondary" size="sm" onClick={onClose}>
            Close
          </Button>
        </ModalFooter>
      </Modal>
    )
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()

    const coercedValues = coerceValues(values, resolvedSchema)

    const validation = validateAgainstSchema(coercedValues, resolvedSchema)

    if (!validation.valid) {
      setErrors(validation.errors)
      return
    }

    const submissionData = extractSchemaFields(coercedValues, resolvedSchema)

    setErrors({})

    setIsSubmitting(true)
    try {
      await onSubmit(tool, submissionData)
      onClose()
    } catch (error) {
      setErrors({
        _general: error instanceof Error ? error.message : 'Failed to execute tool',
      })
    } finally {
      setIsSubmitting(false)
    }
  }

  return (
    <Modal
      isOpen={isOpen}
      onClose={onClose}
      variant="accent"
      size="md"
      closeOnBackdrop={!isSubmitting}
      closeOnEscape={!isSubmitting}
    >
      <form onSubmit={handleSubmit} className="flex flex-col max-h-[90vh]">
        <ModalBody className="flex-1 overflow-y-auto">
          <div className="mb-md">
            <h2 className="text-xl font-heading font-normal uppercase text-text-primary">{tool.name}</h2>
            {tool.description && (
              <p className="text-sm text-text-secondary mt-xs">{tool.description}</p>
            )}
          </div>

          {errors._general && (
            <div className="mb-md p-sm bg-error/10 border border-error/30 rounded-md">
              <p className="text-sm text-error">{errors._general}</p>
            </div>
          )}

          <SchemaFormGenerator
            schema={resolvedSchema}
            values={values}
            onChange={setValues}
            errors={errors}
            onLoadingChange={setIsLoadingData}
          />

          {Object.keys(resolvedSchema.properties || {}).length === 0 && (
            <div className="text-center py-lg text-text-secondary">
              <p className="text-sm">This tool requires no parameters</p>
            </div>
          )}
        </ModalBody>

        <ModalFooter>
          <Button
            type="button"
            variant="ghost"
            size="sm"
            onClick={onClose}
            disabled={isSubmitting || isLoadingData}
          >
            Cancel
          </Button>
          <Button
            type="submit"
            variant="primary"
            size="sm"
            icon={Play}
            iconPosition="left"
            loading={isSubmitting || isLoadingData}
            disabled={isSubmitting || isLoadingData}
          >
            {isSubmitting ? 'Running...' : isLoadingData ? 'Loading...' : 'Run Tool'}
          </Button>
        </ModalFooter>
      </form>
    </Modal>
  )
}
