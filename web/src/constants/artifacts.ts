/**
 * Artifact Types - Categorizes different kinds of artifacts by their content/purpose
 */
export const ArtifactType = {
  TOOL_EXECUTION: 'tool_execution',
  TABLE: 'table',
  CHART: 'chart',
  CODE: 'code',
  FORM: 'form',
  TEXT: 'text',
  FILE: 'file',
  TREE: 'tree',
  CARD: 'card',
  PRESENTATION: 'presentation',
  KNOWLEDGE_QUERY: 'knowledge_query',
} as const

export type ArtifactType = typeof ArtifactType[keyof typeof ArtifactType]

/**
 * Artifact Metadata Fields - Standard metadata keys for all artifacts
 */
export const ArtifactMetadataKey = {
  ARTIFACT_TYPE: 'artifact_type',
  CONTEXT_ID: 'context_id',
  TOOL_EXECUTION_ID: 'tool_execution_id',
  TOOL_NAME: 'tool_name',
  CREATED_AT: 'created_at',
  IS_INTERNAL: 'is_internal',
  RENDERING_HINTS: 'rendering_hints',
  SOURCE: 'source',
} as const

export type ArtifactMetadataKey = typeof ArtifactMetadataKey[keyof typeof ArtifactMetadataKey]

/**
 * Artifact Source - Where the artifact originated
 */
export const ArtifactSource = {
  MCP_TOOL: 'mcp_tool',
  AGENT: 'agent',
  SYSTEM: 'system',
  USER: 'user',
} as const

export type ArtifactSource = typeof ArtifactSource[keyof typeof ArtifactSource]
