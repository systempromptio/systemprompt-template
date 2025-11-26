import { cn } from '@/lib/utils/cn'
import type { LucideIcon } from 'lucide-react'

export type BadgeVariant = 'success' | 'warning' | 'error' | 'primary' | 'secondary' | 'info' | 'muted'
export type BadgeSize = 'xs' | 'sm' | 'md' | 'lg'

interface BadgeProps {
  label: string
  variant?: BadgeVariant
  size?: 'xs' | 'sm' | 'md' | 'lg'
  icon?: LucideIcon
  showIcon?: boolean
  className?: string
}

export function Badge({
  label,
  variant = 'primary',
  size = 'md',
  icon: Icon,
  showIcon = true,
  className
}: BadgeProps) {
  const sizeClasses = {
    xs: 'px-xs py-0.5 text-xs',
    sm: 'px-sm py-xs text-xs',
    md: 'px-sm py-xs text-sm',
    lg: 'px-md py-sm text-base'
  }

  const iconSizes = {
    xs: 'w-3 h-3',
    sm: 'w-3.5 h-3.5',
    md: 'w-4 h-4',
    lg: 'w-5 h-5'
  }

  const variantClasses = {
    success: 'bg-success/20 text-success',
    warning: 'bg-warning/20 text-warning',
    error: 'bg-error/20 text-error',
    primary: 'bg-primary/20 text-primary',
    secondary: 'bg-secondary/20 text-secondary',
    info: 'bg-primary/20 text-primary',
    muted: 'bg-text-disabled/20 text-text-secondary'
  }

  return (
    <div
      className={cn(
        'inline-flex items-center gap-xs rounded-md font-medium transition-fast',
        sizeClasses[size],
        variantClasses[variant],
        className
      )}
    >
      {Icon && showIcon && <Icon className={iconSizes[size]} />}
      <span>{label}</span>
    </div>
  )
}
