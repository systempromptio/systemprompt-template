/**
 * Presentation card call-to-action buttons.
 *
 * Displays CTA buttons with loading and success states.
 *
 * @module artifacts/renderers/presentation/CardCTAs
 */

import React from 'react'
import { ArrowRight, Check, Loader2 } from 'lucide-react'
import { cn } from '@/lib/utils/cn'

interface CTA {
  id: string
  label: string
  message: string
  variant?: 'primary' | 'secondary' | 'outline'
  icon?: string
}

const ICON_MAP: Record<string, React.ComponentType<{ className?: string }>> = {}

function getIconComponent(iconName: string) {
  return ICON_MAP[iconName] || null
}

interface CardCTAsProps {
  ctas: CTA[]
  clickingId: string | null
  successId: string | null
  onCTAClick: (ctaId: string, message: string) => void
}

export const CardCTAs = React.memo(function CardCTAs({ ctas, clickingId, successId, onCTAClick }: CardCTAsProps) {
  const getIcon = (iconName?: string) => {
    if (!iconName) return null
    const Icon = getIconComponent(iconName)
    return Icon ? <Icon className="w-4 h-4" /> : null
  }

  if (!ctas || ctas.length === 0) return null

  return (
    <div className="px-[var(--card-padding-lg)] py-[var(--spacing-lg)] bg-surface-variant/50 border-t border-primary-10">
      <div className="flex flex-wrap gap-[var(--spacing-sm)]">
        {ctas.map((cta) => {
          const variantClasses = {
            primary: cn('bg-primary hover:bg-primary/90 text-text-inverted border-primary', 'shadow-[var(--shadow-sm)] hover:shadow-[var(--shadow-md)]'),
            secondary: cn('bg-surface hover:bg-surface-dark/50 text-primary border-primary-15'),
            outline: cn('bg-transparent hover:bg-surface-dark/20 text-primary border-primary-15'),
          }

          const variant = cta.variant || 'secondary'
          const isClicking = clickingId === cta.id
          const isSuccess = successId === cta.id

          return (
            <button
              key={cta.id}
              onClick={() => onCTAClick(cta.id, cta.message)}
              disabled={isClicking}
              className={cn(
                'group',
                'inline-flex items-center gap-[var(--spacing-sm)]',
                'px-[var(--spacing-lg)] py-[var(--spacing-md)]',
                'border rounded-[var(--radius-lg)]',
                'font-body font-medium text-[var(--font-size-sm)]',
                'transition-all duration-[var(--animation-normal)]',
                'hover:scale-[1.02] active:scale-[0.98]',
                'disabled:opacity-70 disabled:cursor-wait',
                variantClasses[variant]
              )}
            >
              {cta.icon && !isClicking && !isSuccess && <span className="transition-transform duration-[var(--animation-fast)] group-hover:rotate-12">{getIcon(cta.icon)}</span>}
              {isClicking && <Loader2 className="w-4 h-4 animate-spin" />}
              {isSuccess && <Check className="w-4 h-4 text-success" />}
              <span>{cta.label}</span>
              {!isClicking && !isSuccess && <ArrowRight className="w-4 h-4 transition-transform duration-[var(--animation-fast)] group-hover:translate-x-1" />}
            </button>
          )
        })}
      </div>
    </div>
  )
})
