import { Shield, User, UserCircle } from 'lucide-react'
import { Badge, type BadgeVariant } from '@/components/ui'

interface RoleBadgeProps {
  role: string
  size?: 'xs' | 'sm' | 'md' | 'lg'
  showIcon?: boolean
  className?: string
}

export function RoleBadge({ role, size = 'md', showIcon = true, className }: RoleBadgeProps) {
  const roleConfig = getRoleConfig(role)

  return (
    <Badge
      label={roleConfig.label}
      variant={roleConfig.variant}
      icon={roleConfig.icon}
      size={size}
      showIcon={showIcon}
      className={className}
    />
  )
}

function getRoleConfig(role: string) {
  const roleLower = role.toLowerCase()

  if (roleLower === 'admin') {
    return {
      label: 'Admin',
      icon: Shield,
      variant: 'primary' as BadgeVariant
    }
  }

  if (roleLower === 'user') {
    return {
      label: 'User',
      icon: User,
      variant: 'secondary' as BadgeVariant
    }
  }

  return {
    label: 'Guest',
    icon: UserCircle,
    variant: 'muted' as BadgeVariant
  }
}
