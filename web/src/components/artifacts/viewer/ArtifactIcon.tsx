/**
 * Artifact type icon component.
 *
 * Returns the appropriate icon based on artifact type.
 *
 * @module artifacts/viewer/ArtifactIcon
 */

import React from 'react'
import { FileText, Database, File, Table, BarChart3, FileInput, GitBranch, Presentation, LayoutDashboard, List } from 'lucide-react'

interface ArtifactIconProps {
  artifactType: string
}

export const ArtifactIcon = React.memo(function ArtifactIcon({ artifactType }: ArtifactIconProps) {
  switch (artifactType) {
    case 'table':
      return <Table className="w-4 h-4 text-success" />
    case 'chart':
      return <BarChart3 className="w-4 h-4 text-primary" />
    case 'presentation_card':
      return <Presentation className="w-4 h-4 text-primary" />
    case 'dashboard':
      return <LayoutDashboard className="w-4 h-4 text-primary" />
    case 'list':
      return <List className="w-4 h-4 text-primary" />
    case 'form':
      return <FileInput className="w-4 h-4 text-primary" />
    case 'tree':
      return <GitBranch className="w-4 h-4 text-success" />
    case 'markdown':
      return <FileText className="w-4 h-4 text-primary" />
    case 'json':
      return <Database className="w-4 h-4 text-success" />
    default:
      return <File className="w-4 h-4 text-disabled" />
  }
})
