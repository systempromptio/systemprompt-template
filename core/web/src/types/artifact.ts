import type { Artifact as A2AArtifact, Part } from '@a2a-js/sdk'

// =============================================================================
// Artifact Type Enum - Matches Rust: core/crates/shared/models/src/artifacts/types.rs
// =============================================================================

export type ArtifactType =
  | 'text'
  | 'table'
  | 'chart'
  | 'form'
  | 'code'
  | 'tree'
  | 'json'
  | 'markdown'
  | 'dashboard'
  | 'presentation_card'
  | 'list'
  | 'copy_paste_text'
  | 'blog'

// =============================================================================
// Column Types - Matches Rust: core/crates/shared/models/src/artifacts/types.rs
// =============================================================================

export type ColumnType =
  | 'string'
  | 'integer'
  | 'number'
  | 'currency'
  | 'percentage'
  | 'date'
  | 'datetime'
  | 'email'
  | 'boolean'
  | 'link'
  | 'badge'
  | 'thumbnail'

// =============================================================================
// Chart Types - Matches Rust: core/crates/shared/models/src/artifacts/types.rs
// =============================================================================

export type ChartType = 'line' | 'bar' | 'pie' | 'doughnut' | 'area' | 'scatter'
export type AxisType = 'category' | 'linear' | 'logarithmic' | 'time'
export type SortOrder = 'asc' | 'desc'

// =============================================================================
// Table Hints - Matches Rust: core/crates/shared/models/src/artifacts/table/hints.rs
// =============================================================================

export interface TableHints {
  columns?: string[]
  sortable_columns?: string[]
  default_sort?: {
    column: string
    order: SortOrder
  }
  filterable?: boolean
  page_size?: number
  column_types?: Record<string, ColumnType>
  row_click_enabled?: boolean
}

// =============================================================================
// Chart Hints - Matches Rust: core/crates/shared/models/src/artifacts/chart/mod.rs
// =============================================================================

export interface ChartHints {
  chart_type?: ChartType
  title?: string
  x_axis?: { label?: string; type?: AxisType }
  y_axis?: { label?: string; type?: AxisType }
  series?: Array<{ name: string; color?: string }>
}

// =============================================================================
// Form Types
// =============================================================================

export interface FormField {
  name: string
  type: string
  label?: string
  placeholder?: string
  help_text?: string
  required?: boolean
  options?: Array<{ value: string; label: string }> | string[]
  default?: unknown
}

export interface FormHints {
  fields?: FormField[]
  submit_action?: string
  submit_method?: 'GET' | 'POST' | 'PUT' | 'DELETE'
  layout?: 'vertical' | 'horizontal' | 'grid'
}

// =============================================================================
// Code Hints
// =============================================================================

export interface CodeHints {
  language?: string
  theme?: string
  line_numbers?: boolean
  highlight_lines?: number[]
}

// =============================================================================
// Tree Hints
// =============================================================================

export interface TreeHints {
  expandable?: boolean
  default_expanded_levels?: number
  show_icons?: boolean
  icon_map?: Record<string, string>
}

// =============================================================================
// Presentation Card - Matches Rust: core/crates/shared/models/src/artifacts/card/mod.rs
// =============================================================================

export interface PresentationSection {
  heading?: string
  content: string
  icon?: string
}

export interface PresentationCTA {
  id: string
  label: string
  message: string
  variant?: 'primary' | 'secondary' | 'outline'
  icon?: string
}

export interface PresentationHints {
  theme?: 'default' | 'gradient' | 'minimal'
}

export interface PresentationCardData {
  title?: string
  subtitle?: string
  icon?: string
  sections?: PresentationSection[]
  ctas?: PresentationCTA[]
  theme?: 'default' | 'gradient' | 'minimal'
}

// =============================================================================
// Dashboard Types - Matches Rust: core/crates/shared/models/src/artifacts/dashboard/
// =============================================================================

export type LayoutMode = 'vertical' | 'grid' | 'tabs'
export type LayoutWidth = 'full' | 'half' | 'third' | 'twothirds'
export type SectionType = 'metrics_cards' | 'table' | 'chart' | 'timeline' | 'status' | 'list'

export interface SectionLayout {
  width?: LayoutWidth
  order?: number
}

export interface DashboardSection {
  section_id: string
  title?: string
  section_type: SectionType
  data: unknown
  layout?: SectionLayout
}

export interface DashboardHints {
  layout?: LayoutMode
  refreshable?: boolean
  refresh_interval_seconds?: number
  drill_down_enabled?: boolean
}

export interface DashboardData {
  title?: string
  description?: string
  sections: DashboardSection[]
}

// =============================================================================
// Rendering Hints Union
// =============================================================================

export type RenderingHints =
  | TableHints
  | ChartHints
  | FormHints
  | CodeHints
  | TreeHints
  | PresentationHints
  | DashboardHints
  | null

// =============================================================================
// Artifact Metadata - Matches Rust: core/crates/shared/models/src/a2a/artifact_metadata.rs
// =============================================================================

interface BaseArtifactMetadata {
  artifact_type: ArtifactType | string
  created_at: string
  rendering_hints?: RenderingHints
  source?: string
  mcp_execution_id?: string
  mcp_schema?: object
  is_internal?: boolean
  fingerprint?: string
  tool_name?: string
  execution_index?: number
  skill_id?: string
  skill_name?: string
  [k: string]: unknown
}

export interface EphemeralArtifactMetadata extends BaseArtifactMetadata {
  ephemeral: true
  source: 'mcp_tool'
  tool_name: string
  mcp_execution_id: string
}

export interface PersistedArtifactMetadata extends BaseArtifactMetadata {
  ephemeral?: false
  context_id: string
  task_id: string
}

export type ArtifactMetadata = EphemeralArtifactMetadata | PersistedArtifactMetadata

// =============================================================================
// Artifact Types
// =============================================================================

export type Artifact = Omit<A2AArtifact, 'metadata'> & {
  metadata: ArtifactMetadata
}

export type EphemeralArtifact = Omit<A2AArtifact, 'metadata'> & {
  metadata: EphemeralArtifactMetadata
}

export type PersistedArtifact = Omit<A2AArtifact, 'metadata'> & {
  metadata: PersistedArtifactMetadata
}

// =============================================================================
// Streaming State
// =============================================================================

export interface StreamingArtifactState {
  isAppending: boolean
  isComplete: boolean
  previousParts: Part[]
}

// =============================================================================
// Type Guards & Utilities
// =============================================================================

export function isEphemeralArtifact(artifact: Artifact): artifact is EphemeralArtifact {
  return artifact.metadata.ephemeral === true
}

export function isPersistedArtifact(artifact: Artifact): artifact is PersistedArtifact {
  return !artifact.metadata.ephemeral
}

export function validateArtifact(artifact: A2AArtifact): artifact is Artifact {
  if (!artifact.metadata) {
    return false
  }

  const metadata = artifact.metadata as Record<string, unknown>
  return (
    typeof metadata.artifact_type === 'string' &&
    typeof metadata.context_id === 'string' &&
    typeof metadata.created_at === 'string'
  )
}

export function toArtifact(artifact: A2AArtifact): Artifact {
  if (!validateArtifact(artifact)) {
    throw new Error(
      `Invalid artifact: missing required metadata fields. ` +
        `Expected: artifact_type, context_id, created_at. ` +
        `Received: ${JSON.stringify(artifact.metadata)}`
    )
  }
  return artifact
}

export function hasArtifactId(
  artifact: unknown
): artifact is { artifactId: string } & Record<string, unknown> {
  return (
    typeof artifact === 'object' &&
    artifact !== null &&
    'artifactId' in artifact &&
    typeof (artifact as { artifactId: unknown }).artifactId === 'string'
  )
}

export function hasFingerprint(artifact: A2AArtifact): boolean {
  return (
    typeof artifact.metadata === 'object' &&
    artifact.metadata !== null &&
    'fingerprint' in artifact.metadata &&
    typeof (artifact.metadata as { fingerprint: unknown }).fingerprint === 'string'
  )
}

export function getArtifactId(artifact: A2AArtifact): string | undefined {
  if (hasArtifactId(artifact)) {
    return artifact.artifactId
  }
  return undefined
}

export function getFingerprint(artifact: A2AArtifact): string | undefined {
  if (hasFingerprint(artifact)) {
    return (artifact.metadata as { fingerprint: string }).fingerprint
  }
  return undefined
}

// =============================================================================
// Part Type Guards
// =============================================================================

interface TextPart {
  kind: 'text'
  text: string
}

interface DataPart {
  kind: 'data'
  data: Record<string, unknown>
}

interface FilePart {
  kind: 'file'
  file: {
    name: string
    mimeType: string
    bytes?: string
    uri?: string
  }
}

export type ArtifactPart = TextPart | DataPart | FilePart

function isPlainObject(value: unknown): value is Record<string, unknown> {
  return value !== null && typeof value === 'object' && !Array.isArray(value)
}

/**
 * Type guard for text parts.
 */
export function isTextPart(part: unknown): part is TextPart {
  if (!isPlainObject(part)) return false
  return part.kind === 'text' && typeof part.text === 'string'
}

/**
 * Type guard for data parts.
 */
export function isDataPart(part: unknown): part is DataPart {
  if (!isPlainObject(part)) return false
  return part.kind === 'data' && 'data' in part
}

/**
 * Type guard for file parts.
 */
export function isFilePart(part: unknown): part is FilePart {
  if (!isPlainObject(part)) return false
  if (part.kind !== 'file') return false
  const file = (part as Record<string, unknown>).file
  return isPlainObject(file) && typeof file.name === 'string' && typeof file.mimeType === 'string'
}

/**
 * Extract data from artifact, throwing if missing.
 *
 * @example
 * ```typescript
 * // Before:
 * const data = (artifact.parts.find(p => (p as any).kind === 'data') as any)?.data
 *
 * // After:
 * const data = getArtifactData(artifact)
 * ```
 */
export function getArtifactData<T = unknown>(artifact: Artifact): T {
  const dataPart = artifact.parts.find((p): p is DataPart => isDataPart(p))
  if (!dataPart) {
    throw new Error(`Artifact ${artifact.artifactId} has no data part`)
  }
  return dataPart.data as T
}

/**
 * Extract text from artifact, throwing if missing.
 *
 * @example
 * ```typescript
 * // Before:
 * const text = (artifact.parts.find(p => (p as any).kind === 'text') as any)?.text
 *
 * // After:
 * const text = getArtifactText(artifact)
 * ```
 */
export function getArtifactText(artifact: Artifact): string {
  const textPart = artifact.parts.find(isTextPart)
  if (!textPart) {
    throw new Error(`Artifact ${artifact.artifactId} has no text part`)
  }
  return textPart.text
}

/**
 * Safely try to extract data from artifact, returning undefined if missing.
 */
export function tryGetArtifactData<T = unknown>(artifact: Artifact): T | undefined {
  const dataPart = artifact.parts.find((p): p is DataPart => isDataPart(p))
  return dataPart?.data as T | undefined
}

/**
 * Safely try to extract text from artifact, returning undefined if missing.
 */
export function tryGetArtifactText(artifact: Artifact): string | undefined {
  const textPart = artifact.parts.find((p): p is TextPart => isTextPart(p))
  return textPart?.text
}
