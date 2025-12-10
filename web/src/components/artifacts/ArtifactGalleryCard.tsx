import type { Artifact } from '@a2a-js/sdk'
import { Card } from '@/components/ui/Card'
import {
  BarChart,
  Table as TableIcon,
  Code,
  FileText,
  Layout,
  Package,
  Calendar
} from 'lucide-react'
import type { ArtifactMetadata } from '@/types/artifact'

interface ArtifactGalleryCardProps {
  artifact: Artifact
  onClick?: () => void
}

export function ArtifactGalleryCard({ artifact, onClick }: ArtifactGalleryCardProps) {
  const metadata = artifact.metadata as ArtifactMetadata | undefined
  const artifactType = metadata?.artifact_type || 'unknown'

  const getIcon = () => {
    switch (artifactType) {
      case 'table':
        return <TableIcon className="w-8 h-8 text-primary" />
      case 'chart':
        return <BarChart className="w-8 h-8 text-success" />
      case 'code':
        return <Code className="w-8 h-8 text-warning" />
      case 'markdown':
        return <FileText className="w-8 h-8 text-info" />
      case 'presentation_card':
        return <Layout className="w-8 h-8 text-accent" />
      default:
        return <Package className="w-8 h-8 text-text-secondary" />
    }
  }

  const getTypeLabel = () => {
    switch (artifactType) {
      case 'table':
        return 'Table'
      case 'chart':
        return 'Chart'
      case 'code':
        return 'Code'
      case 'markdown':
        return 'Markdown'
      case 'presentation_card':
        return 'Presentation'
      case 'tree':
        return 'Tree'
      case 'json':
        return 'JSON'
      case 'form':
        return 'Form'
      default:
        return 'Artifact'
    }
  }

  const getPreviewText = () => {
    if (!artifact.parts || artifact.parts.length === 0) {
      return 'No content'
    }

    const firstPart = artifact.parts[0]

    if ('text' in firstPart) {
      return firstPart.text.substring(0, 100) + (firstPart.text.length > 100 ? '...' : '')
    }

    if ('data' in firstPart) {
      return 'Structured data'
    }

    if ('file' in firstPart) {
      return firstPart.file.name || 'File attachment'
    }

    return 'Complex artifact'
  }

  return (
    <Card
      variant="accent"
      padding="md"
      elevation="sm"
      className="cursor-pointer hover:shadow-lg transition-shadow"
      onClick={onClick}
    >
      <div className="flex flex-col gap-md h-full">
        <div className="flex items-start justify-between gap-sm">
          <div className="flex-shrink-0">{getIcon()}</div>
          <span className="px-xs py-xs rounded text-xs font-medium bg-surface-variant text-text-secondary">
            {getTypeLabel()}
          </span>
        </div>

        <div className="flex-1">
          <h3 className="font-heading font-semibold text-text-primary mb-xs line-clamp-2">
            {artifact.name || 'Untitled Artifact'}
          </h3>

          {artifact.description && (
            <p className="text-xs text-text-secondary mb-sm line-clamp-2">
              {artifact.description}
            </p>
          )}

          <p className="text-xs text-text-secondary/80 line-clamp-3">
            {getPreviewText()}
          </p>
        </div>

        <div className="flex items-center gap-xs text-xs text-text-secondary pt-sm border-t border-surface-variant">
          <Calendar className="w-3 h-3" />
          <span className="text-xs font-mono text-text-secondary/60">
            {artifact.artifactId.slice(0, 8)}
          </span>
        </div>
      </div>
    </Card>
  )
}
