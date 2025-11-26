import { User } from 'lucide-react'
import { cn } from '@/lib/utils/cn'
import { getInitials, generateColorPalette, getContrastColor } from '@/lib/utils/avatar'

interface AvatarProps {
  variant?: 'user' | 'agent'
  username?: string | null
  email?: string | null
  userId?: string | null
  agentName?: string
  agentId?: string
  size?: 'sm' | 'md' | 'lg'
  showGlow?: boolean
  animated?: boolean
  clickable?: boolean
  onClick?: () => void
  className?: string
}

const sizeMap = {
  sm: {
    container: 'w-10 h-10',
    text: 'text-base',
    icon: 'w-5 h-5',
  },
  md: {
    container: 'w-12 h-12',
    text: 'text-xl',
    icon: 'w-6 h-6',
  },
  lg: {
    container: 'w-16 h-16',
    text: 'text-2xl',
    icon: 'w-8 h-8',
  },
}

export function Avatar({
  variant = 'user',
  username,
  email,
  userId,
  agentName,
  agentId,
  size = 'md',
  showGlow = true,
  animated = true,
  clickable = false,
  onClick,
  className,
}: AvatarProps) {
  const sizeClasses = sizeMap[size]

  // Agent variant
  if (variant === 'agent') {
    const initials = getInitials(agentName, null)
    const colors = generateColorPalette(agentId || agentName || 'agent')
    const textColor = getContrastColor(colors.primary)

    return (
      <div
        className={cn(
          'relative flex-shrink-0 rounded-full flex items-center justify-center',
          'border border-white/20',
          'transition-all duration-normal',
          clickable && 'hover:scale-110 cursor-pointer',
          animated && 'animate-pulseGlow',
          sizeClasses.container,
          className
        )}
        onClick={clickable ? onClick : undefined}
        role={clickable ? 'button' : undefined}
        tabIndex={clickable ? 0 : undefined}
        style={{
          background: `linear-gradient(315deg, ${colors.primary} 0%, ${colors.secondary} 100%)`,
          boxShadow: showGlow
            ? `
              0 4px 16px rgba(${colors.primaryRgb}, 0.3),
              0 2px 8px rgba(${colors.secondaryRgb}, 0.2),
              inset 0 1px 2px rgba(255, 255, 255, 0.15)
            `
            : 'none',
        }}
      >
        {animated && (
          <div
            className="absolute inset-0 animate-shimmerSweep rounded-full overflow-hidden"
            style={{
              background: 'linear-gradient(90deg, transparent 0%, rgba(255, 255, 255, 0.15) 50%, transparent 100%)',
              backgroundSize: '200% 100%',
            }}
          />
        )}

        <span
          className={cn(
            'relative z-10 font-heading select-none',
            sizeClasses.text
          )}
          style={{
            color: textColor,
            textShadow: '0 1px 2px rgba(0, 0, 0, 0.3)',
          }}
        >
          {initials}
        </span>
      </div>
    )
  }

  // User variant
  const isAuthenticated = !!(username || email)
  const initials = getInitials(username, email)
  const colors = generateColorPalette(userId)
  const textColor = getContrastColor(colors.primary)

  if (!isAuthenticated) {
    return (
      <div
        className={cn(
          'flex-shrink-0 rounded-full flex items-center justify-center',
          'bg-surface-dark/20 backdrop-blur-sm',
          'border border-border/30',
          'transition-all duration-normal',
          clickable && 'hover:scale-110 hover:border-primary/40 cursor-pointer',
          sizeClasses.container,
          className
        )}
        onClick={clickable ? onClick : undefined}
        role={clickable ? 'button' : undefined}
        tabIndex={clickable ? 0 : undefined}
      >
        <User className={cn('text-text-secondary', sizeClasses.icon)} />
      </div>
    )
  }

  return (
    <div
      className={cn(
        'relative flex-shrink-0 rounded-full flex items-center justify-center',
        'border border-white/20',
        'transition-all duration-normal',
        clickable && 'hover:scale-110 cursor-pointer',
        animated && 'animate-pulseGlow',
        sizeClasses.container,
        className
      )}
      onClick={clickable ? onClick : undefined}
      role={clickable ? 'button' : undefined}
      tabIndex={clickable ? 0 : undefined}
      style={{
        background: `linear-gradient(135deg, ${colors.primary} 0%, ${colors.secondary} 100%)`,
        boxShadow: showGlow
          ? `
            0 4px 16px rgba(${colors.primaryRgb}, 0.4),
            0 2px 8px rgba(${colors.secondaryRgb}, 0.3),
            inset 0 1px 2px rgba(255, 255, 255, 0.2)
          `
          : 'none',
      }}
    >
      {animated && (
        <div
          className="absolute inset-0 animate-shimmerSweep rounded-full overflow-hidden"
          style={{
            background: 'linear-gradient(90deg, transparent 0%, rgba(255, 255, 255, 0.2) 50%, transparent 100%)',
            backgroundSize: '200% 100%',
          }}
        />
      )}

      <span
        className={cn(
          'relative z-10 font-heading select-none',
          sizeClasses.text
        )}
        style={{
          color: textColor,
          textShadow: '0 1px 2px rgba(0, 0, 0, 0.3)',
        }}
      >
        {initials}
      </span>
    </div>
  )
}
