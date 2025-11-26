import { Lock } from 'lucide-react'
import { cn } from '@/lib/utils/cn'

interface LockedFeatureProps {
  locked: boolean
  children: React.ReactNode
  message?: string
  onClick?: () => void
  className?: string
}

export function LockedFeature({
  locked,
  children,
  message = 'Sign in to access',
  onClick,
  className
}: LockedFeatureProps) {
  if (!locked) {
    return <>{children}</>
  }

  return (
    <div className={cn('relative overflow-hidden rounded-lg', className)}>
      <div className="opacity-40 pointer-events-none grayscale">
        {children}
      </div>
      <div
        className={cn(
          'absolute inset-0 flex items-center justify-center rounded-lg overflow-hidden',
          'bg-gradient-to-br from-surface-dark/40 via-surface-dark/60 to-surface-dark/40',
          'backdrop-blur-md',
          onClick && 'cursor-pointer hover:from-surface-dark/50 hover:via-surface-dark/70 hover:to-surface-dark/50 transition-all duration-300'
        )}
        onClick={onClick}
      >
        <div className="flex flex-col items-center gap-sm text-center p-md max-w-xs">
          <div className="w-12 h-12 rounded-full bg-gradient-to-br from-warning/30 to-warning/10 flex items-center justify-center backdrop-blur-sm border border-warning/20 shadow-lg">
            <Lock className="w-6 h-6 text-warning" />
          </div>
          <span className="text-sm font-medium text-white drop-shadow-lg">{message}</span>
          {onClick && (
            <span className="text-xs text-white/80 drop-shadow">Click to sign in</span>
          )}
        </div>
      </div>
    </div>
  )
}
