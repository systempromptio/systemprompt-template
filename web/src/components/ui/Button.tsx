import React from 'react';
import { Loader2, type LucideIcon } from 'lucide-react';
import { cn } from '@/lib/utils/cn';

export type ButtonVariant = 'primary' | 'secondary' | 'ghost' | 'destructive' | 'success';
export type ButtonSize = 'xs' | 'sm' | 'md' | 'lg';

interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: ButtonVariant;
  size?: ButtonSize;
  icon?: LucideIcon;
  iconPosition?: 'left' | 'right';
  loading?: boolean;
  children?: React.ReactNode;
}

const variantStyles: Record<ButtonVariant, string> = {
  primary: 'bg-primary text-white hover:opacity-90 active:scale-95 shadow-sm',
  secondary: 'border-2 border-primary text-primary hover:bg-primary/10 active:scale-95',
  ghost: 'text-primary hover:bg-primary/10 active:scale-95',
  destructive: 'bg-error text-white hover:opacity-90 active:scale-95 shadow-sm',
  success: 'bg-success text-white hover:opacity-90 active:scale-95 shadow-sm',
};

const sizeStyles: Record<ButtonSize, { button: string; icon: string }> = {
  xs: { button: 'px-xs py-xs text-xs min-h-[36px]', icon: 'w-3 h-3' },
  sm: { button: 'px-sm py-sm text-sm min-h-[40px]', icon: 'w-4 h-4' },
  md: { button: 'px-md py-sm text-md min-h-[44px]', icon: 'w-5 h-5' },
  lg: { button: 'px-lg py-md text-lg min-h-[48px]', icon: 'w-6 h-6' },
};

export const Button: React.FC<ButtonProps> = ({
  variant = 'primary',
  size = 'md',
  icon: Icon,
  iconPosition = 'left',
  loading = false,
  disabled,
  children,
  className,
  ...props
}) => {
  const isDisabled = disabled || loading;
  const iconSize = sizeStyles[size].icon;

  return (
    <button
      disabled={isDisabled}
      className={cn(
        'inline-flex items-center justify-center gap-xs',
        'rounded-md font-medium font-body',
        'transition-all duration-fast',
        'focus:outline-none focus:ring-2 focus:ring-primary focus:ring-offset-2',
        'disabled:opacity-50 disabled:cursor-not-allowed disabled:transform-none',
        variantStyles[variant],
        sizeStyles[size].button,
        className
      )}
      {...props}
    >
      {loading && <Loader2 className={cn(iconSize, 'animate-spin')} />}
      {!loading && Icon && iconPosition === 'left' && <Icon className={iconSize} />}
      {children}
      {!loading && Icon && iconPosition === 'right' && <Icon className={iconSize} />}
    </button>
  );
};

export default Button;
