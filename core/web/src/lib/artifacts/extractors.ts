import type { Artifact, ArtifactMetadata, PresentationCardData, DashboardData } from '@/types/artifact'
import type { ChartData, TreeNode } from '@/lib/mcp/types'
import { isArrayResponse, hasLabelsAndDatasets, hasNameProperty } from '@/lib/mcp/types'

export interface ExtractionResult<T> {
  data: T
  errors?: string[]
}

interface ToolResponseWrapper {
  artifact_id?: string
  artifact?: unknown
  _metadata?: unknown
}

function isToolResponseWrapper(data: unknown): data is ToolResponseWrapper {
  return (
    typeof data === 'object' &&
    data !== null &&
    'artifact' in data &&
    (data as ToolResponseWrapper).artifact !== undefined
  )
}

function unwrapToolResponse(data: unknown): unknown {
  if (isToolResponseWrapper(data)) {
    return data.artifact
  }
  return data
}

export function unwrapExtraction<T>(result: ExtractionResult<T>): T {
  return result.data
}

export const extractMetadata = (artifact: Artifact): ArtifactMetadata => {
  return artifact.metadata
}

export const extractTableData = (artifact: Artifact): ExtractionResult<unknown[]> => {
  const dataPart = artifact.parts.find(p => p.kind === 'data')
  if (!dataPart || dataPart.kind !== 'data') {
    return { data: [], errors: ['No data part found in artifact'] }
  }

  const rawData = dataPart.data
  const data = unwrapToolResponse(rawData)
  const validationErrors = artifact.metadata.validation_errors as string[] | undefined

  if (isArrayResponse(data)) {
    return { data: data.items, errors: validationErrors }
  }

  return {
    data: [],
    errors: ['Data must be in {items: [...]} format per MCP spec']
  }
}

export const extractChartData = (artifact: Artifact): ExtractionResult<ChartData | null> => {
  const dataPart = artifact.parts.find(p => p.kind === 'data')
  if (!dataPart || dataPart.kind !== 'data') {
    return { data: null, errors: ['No data part found'] }
  }

  const rawData = dataPart.data
  const data = unwrapToolResponse(rawData)
  const validationErrors = artifact.metadata.validation_errors as string[] | undefined

  if (hasLabelsAndDatasets(data)) {
    return { data, errors: validationErrors }
  }

  return { data: null, errors: ['Data must have labels and datasets'] }
}

export const extractCodeData = (artifact: Artifact): ExtractionResult<string> => {
  const dataPart = artifact.parts.find(p => p.kind === 'data')
  const validationErrors = artifact.metadata.validation_errors as string[] | undefined

  if (dataPart && dataPart.kind === 'data') {
    const rawData = dataPart.data
    const data = unwrapToolResponse(rawData)
    if (typeof data === 'string') {
      return { data, errors: validationErrors }
    }
    if (typeof data === 'object') {
      return { data: JSON.stringify(data, null, 2), errors: validationErrors }
    }
  }

  const textPart = artifact.parts.find(p => p.kind === 'text')
  if (textPart && textPart.kind === 'text') {
    return { data: textPart.text }
  }

  return { data: '', errors: ['No code data found'] }
}

export const extractTreeData = (artifact: Artifact): ExtractionResult<TreeNode | null> => {
  const dataPart = artifact.parts.find(p => p.kind === 'data')
  if (!dataPart || dataPart.kind !== 'data') {
    return { data: null, errors: ['No data part found'] }
  }

  const rawData = dataPart.data
  const data = unwrapToolResponse(rawData)
  const validationErrors = artifact.metadata.validation_errors as string[] | undefined

  if (hasNameProperty(data)) {
    return { data, errors: validationErrors }
  }

  return { data: null, errors: ['Data must be tree structure with name property'] }
}

export const extractPresentationCardData = (artifact: Artifact): ExtractionResult<PresentationCardData> => {
  const dataPart = artifact.parts.find(p => p.kind === 'data')
  if (!dataPart || dataPart.kind !== 'data') {
    return {
      data: {},
      errors: ['No data part found in artifact']
    }
  }

  const rawData = dataPart.data
  const data = unwrapToolResponse(rawData) as PresentationCardData
  const validationErrors = artifact.metadata.validation_errors as string[] | undefined

  return {
    data,
    errors: validationErrors
  }
}

export const extractDashboardData = (artifact: Artifact): ExtractionResult<DashboardData | null> => {
  const dataPart = artifact.parts.find(p => p.kind === 'data')
  if (!dataPart || dataPart.kind !== 'data') {
    return {
      data: null,
      errors: ['No data part found in artifact']
    }
  }

  const rawData = dataPart.data
  const data = unwrapToolResponse(rawData) as Record<string, unknown>
  const validationErrors = artifact.metadata.validation_errors as string[] | undefined

  if (!data.sections || !Array.isArray(data.sections)) {
    return {
      data: null,
      errors: ['Dashboard data must have sections array']
    }
  }

  return {
    data: {
      title: data.title as string | undefined,
      description: data.description as string | undefined,
      sections: data.sections as DashboardData['sections'],
    },
    errors: validationErrors
  }
}

export interface ListItem {
  title: string
  summary: string
  link: string
}

export interface ListData {
  items: ListItem[]
  count?: number
}

export const extractListData = (artifact: Artifact): ExtractionResult<ListData> => {
  const dataPart = artifact.parts.find(p => p.kind === 'data')
  if (!dataPart || dataPart.kind !== 'data') {
    return {
      data: { items: [] },
      errors: ['No data part found in artifact']
    }
  }

  const rawData = dataPart.data
  const data = unwrapToolResponse(rawData)
  const validationErrors = artifact.metadata.validation_errors as string[] | undefined

  if (isArrayResponse(data)) {
    return {
      data: {
        items: data.items as ListItem[],
        count: (data as unknown as Record<string, unknown>).count as number | undefined
      },
      errors: validationErrors
    }
  }

  return {
    data: { items: [] },
    errors: ['List data must be in {items: [...]} format per MCP spec']
  }
}
