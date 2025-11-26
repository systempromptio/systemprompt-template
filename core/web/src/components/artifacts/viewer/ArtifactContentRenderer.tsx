/**
 * Artifact content renderer.
 *
 * Routes to appropriate renderer based on artifact type.
 *
 * @module artifacts/viewer/ArtifactContentRenderer
 */

import React, { lazy, Suspense } from 'react'
import type { Artifact } from '@/types/artifact'
import type { TableHints, ChartHints, FormHints, TreeHints, PresentationHints, DashboardHints } from '@/types/artifacts'
import { TextArtifact } from '../TextArtifact'
import { FileArtifact } from '../FileArtifact'
import { DataArtifact } from '../DataArtifact'
import { LoadingFallback } from './LoadingFallback'

const TableRenderer = lazy(() => import('../renderers/TableRenderer').then(m => ({ default: m.TableRenderer })))
const ChartRenderer = lazy(() => import('../renderers/ChartRenderer').then(m => ({ default: m.ChartRenderer })))
const DashboardRenderer = lazy(() => import('../renderers/DashboardRenderer').then(m => ({ default: m.DashboardRenderer })))
const FormRenderer = lazy(() => import('../renderers/FormRenderer').then(m => ({ default: m.FormRenderer })))
const TreeRenderer = lazy(() => import('../renderers/TreeRenderer').then(m => ({ default: m.TreeRenderer })))
const PresentationCardRenderer = lazy(() => import('../renderers/PresentationCardRenderer').then(m => ({ default: m.PresentationCardRenderer })))
const KnowledgeQueryRenderer = lazy(() => import('../renderers/KnowledgeQueryRenderer').then(m => ({ default: m.KnowledgeQueryRenderer })))
const ListRenderer = lazy(() => import('../renderers/ListRenderer').then(m => ({ default: m.ListRenderer })))
const CopyPasteTextRenderer = lazy(() => import('../renderers/CopyPasteTextRenderer').then(m => ({ default: m.CopyPasteTextRenderer })))
const BlogRenderer = lazy(() => import('../renderers/BlogRenderer').then(m => ({ default: m.BlogRenderer })))

interface ArtifactContentRendererProps {
  artifact: Artifact
  artifactType: string
  toolName?: string
  metadata: { artifact_type?: string; rendering_hints?: unknown; tool_name?: string; source?: string } | null
}

export const ArtifactContentRenderer = React.memo(function ArtifactContentRenderer({
  artifact,
  artifactType,
  toolName,
  metadata,
}: ArtifactContentRendererProps) {
  if (toolName === 'query_knowledge') {
    return (
      <Suspense fallback={<LoadingFallback />}>
        <KnowledgeQueryRenderer artifact={artifact} />
      </Suspense>
    )
  }

  switch (artifactType) {
    case 'table':
      return (
        <Suspense fallback={<LoadingFallback />}>
          <TableRenderer artifact={artifact} hints={(metadata?.rendering_hints || {}) as TableHints} />
        </Suspense>
      )

    case 'chart':
      return (
        <Suspense fallback={<LoadingFallback />}>
          <ChartRenderer artifact={artifact} hints={(metadata?.rendering_hints || {}) as ChartHints} />
        </Suspense>
      )

    case 'presentation_card':
      return (
        <Suspense fallback={<LoadingFallback />}>
          <PresentationCardRenderer artifact={artifact} hints={(metadata?.rendering_hints || {}) as PresentationHints} />
        </Suspense>
      )

    case 'form':
      return (
        <Suspense fallback={<LoadingFallback />}>
          <FormRenderer hints={(metadata?.rendering_hints || {}) as FormHints} />
        </Suspense>
      )

    case 'tree':
      return (
        <Suspense fallback={<LoadingFallback />}>
          <TreeRenderer artifact={artifact} hints={(metadata?.rendering_hints || {}) as TreeHints} />
        </Suspense>
      )

    case 'dashboard':
      return (
        <Suspense fallback={<LoadingFallback />}>
          <DashboardRenderer artifact={artifact} hints={(metadata?.rendering_hints || {}) as DashboardHints} />
        </Suspense>
      )

    case 'list':
      return (
        <Suspense fallback={<LoadingFallback />}>
          <ListRenderer artifact={artifact} />
        </Suspense>
      )

    case 'copy_paste_text':
      return (
        <Suspense fallback={<LoadingFallback />}>
          <CopyPasteTextRenderer artifact={artifact} />
        </Suspense>
      )

    case 'blog':
      return (
        <Suspense fallback={<LoadingFallback />}>
          <BlogRenderer artifact={artifact} />
        </Suspense>
      )

    case 'markdown':
      return (
        <div>
          {artifact.parts.map((part, idx) => (
            <div key={idx} className="mt-2 first:mt-0">
              {part.kind === 'text' && <TextArtifact part={part} />}
            </div>
          ))}
        </div>
      )

    case 'json':
      return (
        <div>
          {artifact.parts.map((part, idx) => (
            <div key={idx} className="mt-2 first:mt-0">
              {part.kind === 'data' && <DataArtifact part={part} />}
            </div>
          ))}
        </div>
      )

    default:
      return (
        <div>
          {artifact.parts.map((part, idx) => (
            <div key={idx} className="mt-2 first:mt-0">
              {part.kind === 'text' && <TextArtifact part={part} />}
              {part.kind === 'file' && <FileArtifact part={part} />}
              {part.kind === 'data' && <DataArtifact part={part} />}
            </div>
          ))}
        </div>
      )
  }
})
