import { Lock } from 'lucide-react'
import { cn } from '@/lib/utils/cn'

interface AuthRequiredBadgeProps {
  size?: 'sm' | 'md' | 'lg'
  variant?: 'warning' | 'error' | 'info'
  label?: string
  className?: string
}

export function AuthRequiredBadge({
  size = 'md',
  variant = 'warning',
  label = 'Auth Required',
  className
}: AuthRequiredBadgeProps) {
  const sizeClasses = {
    sm: 'px-xs py-0.5 text-xs',
    md: 'px-sm py-xs text-sm',
    lg: 'px-md py-sm text-base'
  }

  const iconSizes = {
    sm: 'w-3 h-3',
    md: 'w-3.5 h-3.5',
    lg: 'w-4 h-4'
  }

  const variantClasses = {
    warning: 'bg-warning/20 text-warning',
    error: 'bg-error/20 text-error',
    info: 'bg-primary/20 text-primary'
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
      <Lock className={iconSizes[size]} />
      <span>{label}</span>
    </div>
  )
}
