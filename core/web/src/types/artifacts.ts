export type ArtifactType = 'table' | 'chart' | 'form' | 'code' | 'tree' | 'json' | 'markdown' | 'presentation_card' | 'dashboard' | 'list' | 'copy_paste_text' | 'blog'

export type ColumnType = 'string' | 'integer' | 'number' | 'currency' | 'percentage' |
                         'datetime' | 'email' | 'badge' | 'link'

export interface TableHints {
  columns?: string[]
  sortable_columns?: string[]
  default_sort?: {
    column: string
    order: 'asc' | 'desc'
  }
  filterable?: boolean
  page_size?: number
  column_types?: Record<string, ColumnType>
}

export interface ChartHints {
  chart_type?: 'bar' | 'line' | 'pie' | 'area' | 'scatter'
  title?: string
  x_axis?: { label?: string; type?: 'category' | 'linear' }
  y_axis?: { label?: string; type?: 'linear' | 'logarithmic' }
  series?: Array<{ name: string; color?: string }>
}

export interface FormHints {
  fields?: FormField[]
  submit_action?: string
  submit_method?: 'GET' | 'POST' | 'PUT' | 'DELETE'
  layout?: 'vertical' | 'horizontal' | 'grid'
}

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

export interface CodeHints {
  language?: string
  theme?: string
  line_numbers?: boolean
  highlight_lines?: number[]
}

export interface TreeHints {
  expandable?: boolean
  default_expanded_levels?: number
  show_icons?: boolean
  icon_map?: Record<string, string>
}

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

// HINTS: Rendering preferences from schema (x-presentation-hints)
export interface PresentationHints {
  theme?: 'default' | 'gradient' | 'minimal'
}

// DATA: Actual content from structured_content in artifact.parts
export interface PresentationCardData {
  title?: string
  subtitle?: string
  icon?: string
  sections?: PresentationSection[]
  ctas?: PresentationCTA[]
  theme?: 'default' | 'gradient' | 'minimal'
}

export interface DashboardHints {
  layout?: 'vertical' | 'horizontal' | 'grid'
}

export interface DashboardSection {
  section_id: string
  title?: string
  section_type: 'metrics_cards' | 'chart' | 'table' | 'list' | 'status'
  data: unknown
  layout?: {
    width?: 'full' | 'half' | 'third'
    order?: number
  }
}

export interface DashboardData {
  title?: string
  description?: string
  sections: DashboardSection[]
}

export type RenderingHints = TableHints | ChartHints | FormHints | CodeHints | TreeHints | PresentationHints | DashboardHints | null

export interface ArtifactMetadata {
  artifact_type: ArtifactType
  rendering_hints?: RenderingHints
  source?: string
  mcp_schema?: object
  mcp_execution_id?: string
  is_internal?: boolean
  fingerprint?: string
  tool_name?: string
}
