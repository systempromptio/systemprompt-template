/**
 * Presentation card header.
 *
 * Displays title, subtitle, and optional icon.
 *
 * @module artifacts/renderers/presentation/CardHeader
 */

import React from 'react'
import { HelpCircle, Info, List, Star, Clock, BookOpen, User, MessageSquare, Zap, MessageCircle, Library, Rocket } from 'lucide-react'

interface CardData {
  icon?: string
  title?: string
  subtitle?: string
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

interface CardHeaderProps {
  cardData: CardData
}

export const CardHeader = React.memo(function CardHeader({ cardData }: CardHeaderProps) {
  const getIcon = (iconName?: string) => {
    if (!iconName) return null
    const Icon = getIconComponent(iconName)
    return <Icon className="w-5 h-5" />
  }

  return (
    <div className="px-[var(--card-padding-lg)] py-[var(--spacing-lg)] border-b border-primary-10">
      <div className="flex items-start gap-[var(--spacing-md)]">
        {cardData.icon && (
          <div className="flex-shrink-0 p-[var(--spacing-md)] bg-primary/10 rounded-[var(--radius-lg)] transition-all duration-[var(--animation-fast)]">
            <div className="text-primary">{getIcon(cardData.icon)}</div>
          </div>
        )}
        <div className="flex-1">
          {cardData.title && (
            <h2 className="font-heading text-3xl font-bold text-primary mb-[var(--spacing-sm)]">
              {cardData.title}
            </h2>
          )}
          {cardData.subtitle && (
            <p className="font-body text-lg text-text-secondary leading-relaxed">
              {cardData.subtitle}
            </p>
          )}
        </div>
      </div>
    </div>
  )
})
