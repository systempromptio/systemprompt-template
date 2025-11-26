import { Check } from 'lucide-react'
import { Card } from './Card'
import { cn } from '@/lib/utils/cn'

interface SelectableCardProps {
  selected?: boolean
  disabled?: boolean
  onClick?: () => void
  children: React.ReactNode
  className?: string
  bordered?: boolean
}

export function SelectableCard({ selected, disabled, onClick, children, className, bordered = true }: SelectableCardProps) {
  return (
    <Card
      variant="accent"
      padding="none"
      elevation="sm"
      bordered={bordered}
      className={cn(
        'relative transition-all duration-fast',
        selected && 'ring-2 ring-success',
        disabled && 'opacity-40 grayscale pointer-events-none',
        !disabled && !selected && 'hover:border-primary/60 hover:scale-105',
        onClick && !disabled && 'cursor-pointer',
        className
      )}
    >
      <div
        onClick={onClick}
        className="w-full text-left relative"
      >
        {children}

        {/* Active indicator - corner badge */}
        {selected && !disabled && (
          <div className="absolute -top-2 -right-2 pointer-events-none">
            <div className="flex items-center justify-center w-6 h-6 bg-success/90 rounded-full shadow-md">
              <Check className="w-4 h-4 text-white" />
            </div>
          </div>
        )}
      </div>
    </Card>
  )
}

interface SelectableCardHeaderProps {
  title: string
  subtitle?: string
  icon?: React.ReactNode
  titleClassName?: string
  subtitleClassName?: string
}

export function SelectableCardHeader({ title, subtitle, icon, titleClassName, subtitleClassName }: SelectableCardHeaderProps) {
  return (
    <div className="flex items-start gap-sm p-sm">
      {icon && (
        <div className="flex-shrink-0 mt-xs">
          {icon}
        </div>
      )}
      <div className="flex-1 min-w-0">
        <div className={cn("font-heading text-sm uppercase tracking-wide text-text-primary truncate", titleClassName)}>
          {title}
        </div>
        {subtitle && (
          <div className={cn("text-xs text-text-secondary mt-xs truncate font-body", subtitleClassName)}>
            {subtitle}
          </div>
        )}
      </div>
    </div>
  )
}

interface SelectableCardContentProps {
  children: React.ReactNode
  className?: string
}

export function SelectableCardContent({ children, className }: SelectableCardContentProps) {
  return (
    <div className={cn("px-sm pb-sm text-sm text-text-secondary font-body line-clamp-2 leading-relaxed", className)}>
      {children}
    </div>
  )
}
