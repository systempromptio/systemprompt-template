import React from 'react';
import { type LucideIcon } from 'lucide-react';
import { cn } from '@/lib/utils/cn';

export type IconSize = 'xs' | 'sm' | 'md' | 'lg' | 'xl';
export type IconColor = 'primary' | 'secondary' | 'success' | 'warning' | 'error' | 'muted' | 'current';

interface IconProps {
  icon: LucideIcon;
  size?: IconSize;
  color?: IconColor;
  className?: string;
}

const sizeStyles: Record<IconSize, string> = {
  xs: 'w-3 h-3',
  sm: 'w-4 h-4',
  md: 'w-5 h-5',
  lg: 'w-6 h-6',
  xl: 'w-8 h-8',
};

const colorStyles: Record<IconColor, string> = {
  primary: 'text-primary',
  secondary: 'text-secondary',
  success: 'text-success',
  warning: 'text-warning',
  error: 'text-error',
  muted: 'text-text-secondary',
  current: 'text-current',
};

export const Icon: React.FC<IconProps> = ({
  icon: LucideIconComponent,
  size = 'md',
  color = 'current',
  className,
}) => {
  return (
    <LucideIconComponent
      className={cn(
        sizeStyles[size],
        colorStyles[color],
        className
      )}
    />
  );
};

export default Icon;
