import { Copy, Check, AlertTriangle, Calendar, FolderOpen, ImageIcon, ExternalLink, User, FileText, Hash, Code, Eye } from 'lucide-react'
import { useState } from 'react'
import type { Artifact } from '@/types/artifact'
import { MarkdownContent } from '@/components/markdown/MarkdownContent'

type ContentView = 'markdown' | 'html'

interface BlogRendererProps {
  artifact: Artifact
}

export function BlogRenderer({ artifact }: BlogRendererProps) {
  const [copied, setCopied] = useState(false)
  const [contentView, setContentView] = useState<ContentView>('markdown')

  const dataPart = artifact.parts.find(p => p.kind === 'data')
  if (!dataPart || dataPart.kind !== 'data') {
    return (
      <div className="flex items-center gap-3 p-4 bg-error/10 border border-error/20 rounded-lg">
        <AlertTriangle className="w-5 h-5 text-error flex-shrink-0" />
        <span className="text-sm text-error">No blog data found</span>
      </div>
    )
  }

  const data = dataPart.data as Record<string, unknown>
  const title = data.title as string
  const slug = data.slug as string
  const content = data.content as string
  const excerpt = data.excerpt as string | undefined
  const featuredImageUrl = data.featured_image_url as string | undefined
  const publishedAt = data.published_at as string | undefined
  const categories = data.categories as string[] | undefined
  const keywords = data.keywords as string | undefined
  const author = data.author as string | undefined
  const contentType = data.content_type as string | undefined

  if (!title || !slug || !content) {
    return (
      <div className="flex items-center gap-3 p-4 bg-error/10 border border-error/20 rounded-lg">
        <AlertTriangle className="w-5 h-5 text-error flex-shrink-0" />
        <span className="text-sm text-error">Invalid blog post data</span>
      </div>
    )
  }

  const handleCopyFrontmatter = () => {
    const frontmatter = generateFrontmatter({
      title,
      slug,
      excerpt,
      publishedAt,
      categories,
      featuredImageUrl,
      keywords,
      author,
      contentType,
    })
    navigator.clipboard.writeText(frontmatter + '\n\n' + content)
    setCopied(true)
    setTimeout(() => setCopied(false), 2000)
  }

  const handleCopyContent = () => {
    navigator.clipboard.writeText(content)
    setCopied(true)
    setTimeout(() => setCopied(false), 2000)
  }

  return (
    <div className="space-y-4">
      {/* Header */}
      <div className="border border-primary-10 rounded-lg overflow-hidden bg-surface">
        <div className="px-4 py-4 bg-surface-variant border-b border-primary-10">
          <h2 className="text-lg font-semibold text-primary mb-1">{title}</h2>
          <div className="flex items-center gap-2">
            <p className="text-sm text-secondary">/{slug}</p>
            <a
              href={`/blog/${slug}`}
              target="_blank"
              rel="noopener noreferrer"
              className="flex items-center gap-1 text-xs text-accent hover:text-accent-dark transition-colors"
            >
              <ExternalLink className="w-3 h-3" />
              <span>Read Content</span>
            </a>
          </div>
        </div>

        {/* Metadata Grid */}
        <div className="px-4 py-4 grid grid-cols-2 gap-4">
          {/* Published At */}
          {publishedAt && (
            <div>
              <p className="text-xs text-secondary mb-1 flex items-center gap-1">
                <Calendar className="w-3 h-3" /> Published
              </p>
              <p className="text-sm text-primary">{new Date(publishedAt).toLocaleDateString()}</p>
            </div>
          )}

          {/* Author */}
          {author && (
            <div>
              <p className="text-xs text-secondary mb-1 flex items-center gap-1">
                <User className="w-3 h-3" /> Author
              </p>
              <p className="text-sm text-primary">{author}</p>
            </div>
          )}

          {/* Content Type */}
          {contentType && (
            <div>
              <p className="text-xs text-secondary mb-1 flex items-center gap-1">
                <FileText className="w-3 h-3" /> Type
              </p>
              <p className="text-sm text-primary capitalize">{contentType}</p>
            </div>
          )}

          {/* Keywords */}
          {keywords && (
            <div className="col-span-2">
              <p className="text-xs text-secondary mb-1 flex items-center gap-1">
                <Hash className="w-3 h-3" /> Keywords
              </p>
              <p className="text-sm text-primary">{keywords}</p>
            </div>
          )}

          {/* Categories */}
          {categories && categories.length > 0 && (
            <div className="col-span-2">
              <p className="text-xs text-secondary mb-2 flex items-center gap-1">
                <FolderOpen className="w-3 h-3" /> Categories
              </p>
              <div className="flex flex-wrap gap-2">
                {categories.map(cat => (
                  <span
                    key={cat}
                    className="inline-flex px-2 py-1 bg-accent/10 text-accent rounded text-xs"
                  >
                    {cat}
                  </span>
                ))}
              </div>
            </div>
          )}

          {/* Featured Image */}
          {featuredImageUrl && (
            <div className="col-span-2">
              <p className="text-xs text-secondary mb-2 flex items-center gap-1">
                <ImageIcon className="w-3 h-3" /> Featured Image
              </p>
              <p className="text-sm text-primary break-all">{featuredImageUrl}</p>
            </div>
          )}

          {/* Excerpt */}
          {excerpt && (
            <div className="col-span-2">
              <p className="text-xs text-secondary mb-1">Excerpt</p>
              <p className="text-sm text-primary italic">{excerpt}</p>
            </div>
          )}
        </div>
      </div>

      {/* Copy Buttons */}
      <div className="flex gap-2">
        <button
          onClick={handleCopyFrontmatter}
          className="flex items-center gap-2 px-3 py-2 bg-primary text-white rounded-md hover:bg-primary-dark transition-colors text-sm font-medium"
          title="Copy with frontmatter"
        >
          {copied ? (
            <>
              <Check className="w-4 h-4" />
              <span>Copied!</span>
            </>
          ) : (
            <>
              <Copy className="w-4 h-4" />
              <span>Copy with Frontmatter</span>
            </>
          )}
        </button>

        <button
          onClick={handleCopyContent}
          className="flex items-center gap-2 px-3 py-2 bg-surface border border-primary-10 text-primary rounded-md hover:bg-surface-variant transition-colors text-sm font-medium"
          title="Copy content only"
        >
          <Copy className="w-4 h-4" />
          <span>Copy Content Only</span>
        </button>
      </div>

      {/* Content Preview with Tabs */}
      <div className="border border-primary-10 rounded-lg overflow-hidden bg-surface">
        <div className="px-4 py-3 bg-surface-variant border-b border-primary-10 flex items-center justify-between">
          <p className="text-sm font-medium text-primary">Content ({content.length} characters)</p>
          <div className="flex items-center gap-1 bg-surface rounded-md p-1">
            <button
              onClick={() => setContentView('markdown')}
              className={`flex items-center gap-1.5 px-3 py-1.5 rounded text-xs font-medium transition-colors ${
                contentView === 'markdown'
                  ? 'bg-primary text-white'
                  : 'text-secondary hover:text-primary hover:bg-surface-variant'
              }`}
            >
              <Code className="w-3.5 h-3.5" />
              Markdown
            </button>
            <button
              onClick={() => setContentView('html')}
              className={`flex items-center gap-1.5 px-3 py-1.5 rounded text-xs font-medium transition-colors ${
                contentView === 'html'
                  ? 'bg-primary text-white'
                  : 'text-secondary hover:text-primary hover:bg-surface-variant'
              }`}
            >
              <Eye className="w-3.5 h-3.5" />
              Preview
            </button>
          </div>
        </div>
        <div className="p-4 overflow-x-auto max-h-[600px] overflow-y-auto">
          {contentView === 'markdown' ? (
            <pre className="text-xs font-mono whitespace-pre-wrap break-words bg-surface-dark p-3 rounded text-primary">
              {content}
            </pre>
          ) : (
            <div className="prose prose-sm max-w-none">
              <MarkdownContent content={content} />
            </div>
          )}
        </div>
      </div>
    </div>
  )
}

function generateFrontmatter(data: {
  title: string
  slug: string
  excerpt?: string
  publishedAt?: string
  categories?: string[]
  featuredImageUrl?: string
  keywords?: string
  author?: string
  contentType?: string
}): string {
  const lines: string[] = ['---']

  lines.push(`title: "${data.title}"`)
  lines.push(`slug: "${data.slug}"`)

  if (data.excerpt) {
    lines.push(`excerpt: "${data.excerpt}"`)
  }

  if (data.keywords) {
    lines.push(`keywords: "${data.keywords}"`)
  }

  if (data.author) {
    lines.push(`author: "${data.author}"`)
  }

  if (data.contentType) {
    lines.push(`type: ${data.contentType}`)
  }

  if (data.publishedAt) {
    lines.push(`published_at: ${data.publishedAt}`)
  }

  if (data.featuredImageUrl) {
    lines.push(`featured_image_url: "${data.featuredImageUrl}"`)
  }

  if (data.categories && data.categories.length > 0) {
    lines.push(`categories: [${data.categories.map(c => `"${c}"`).join(', ')}]`)
  }

  lines.push('---')

  return lines.join('\n')
}
