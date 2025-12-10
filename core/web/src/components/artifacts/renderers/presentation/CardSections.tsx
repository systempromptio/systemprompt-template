/**
 * Presentation card sections.
 *
 * Displays multiple content sections with icons.
 *
 * @module artifacts/renderers/presentation/CardSections
 */

import React from 'react'
import { HelpCircle, Info, List, Star, Clock, BookOpen, User, MessageSquare, Zap, MessageCircle, Library, Rocket } from 'lucide-react'
import { MarkdownContent } from '@/components/markdown/MarkdownContent'

interface Section {
  icon?: string
  heading?: string
  content: string
}

const ICON_MAP: Record<string, React.ComponentType<{ className?: string }>> = {
  info: Info,
  list: List,
  star: Star,
  clock: Clock,
  bookopen: BookOpen,
  user: User,
  messagesquare: MessageSquare,
  zap: Zap,
  messagecircle: MessageCircle,
  library: Library,
  rocket: Rocket,
}

function getIconComponent(iconName: string) {
  const normalizedName = iconName.toLowerCase()
  return ICON_MAP[normalizedName] || HelpCircle
}

interface CardSectionsProps {
  sections: Section[]
}

export const CardSections = React.memo(function CardSections({ sections }: CardSectionsProps) {
  const getIcon = (iconName?: string) => {
    if (!iconName) return null
    const Icon = getIconComponent(iconName)
    return <Icon className="w-5 h-5" />
  }

  if (!sections || sections.length === 0) return null

  return (
    <div className="px-[var(--card-padding-lg)] py-[var(--spacing-lg)] space-y-[var(--spacing-lg)]">
      {sections.map((section, idx) => (
        <div key={idx} className="flex items-start gap-[var(--spacing-md)] group">
          {section.icon && (
            <div className="flex-shrink-0 mt-1 text-primary group-hover:scale-110 transition-transform duration-[var(--animation-fast)]">
              {getIcon(section.icon)}
            </div>
          )}
          <div className="flex-1">
            {section.heading && (
              <h3 className="font-heading text-xl font-semibold text-primary mb-[var(--spacing-sm)]">
                {section.heading}
              </h3>
            )}
            <div className="font-body text-text-secondary leading-relaxed">
              <MarkdownContent content={section.content} />
            </div>
          </div>
        </div>
      ))}
    </div>
  )
})
