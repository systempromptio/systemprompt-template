import React from 'react';
import { cn } from '@/lib/utils/cn';

interface CardProps {
  children: React.ReactNode;
  variant?: 'default' | 'accent' | 'dark' | 'glass' | 'error' | 'success';
  padding?: 'none' | 'sm' | 'md' | 'lg';
  elevation?: 'none' | 'sm' | 'md' | 'lg';
  bordered?: boolean;
  cutCorner?: 'top-left' | 'top-right';
  className?: string;
  onClick?: () => void;
}

/**
 * Card component using theme CSS variables
 * Features: cut corner, theme gradients, theme borders, shadows
 */
export const Card: React.FC<CardProps> = ({
  children,
  variant = 'accent',
  padding = 'md',
  elevation = 'sm',
  bordered = true,
  cutCorner = 'top-right',
  className = '',
  onClick,
}) => {
  const getVariantClasses = () => {
    switch (variant) {
      case 'accent':
        return cn(
          'bg-gradient-to-br',
          'from-[var(--card-gradient-start)] via-[var(--card-gradient-mid)] to-[var(--card-gradient-end)]',
          bordered && 'border border-primary'
        );

      case 'glass':
        return cn(
          'bg-gradient-to-br',
          'from-[rgba(var(--color-primary-rgb),0.08)] via-[rgba(var(--color-primary-rgb),0.05)] to-[rgba(var(--color-primary-rgb),0.03)]',
          'backdrop-blur-md',
          bordered && 'border border-primary/50'
        );

      case 'dark':
        return cn(
          'bg-surface-dark',
          bordered && 'border border-primary/25'
        );

      case 'error':
        return cn(
          'bg-error/10',
          bordered && 'border border-error/40'
        );

      case 'success':
        return cn(
          'bg-success/10',
          bordered && 'border border-success/40'
        );

      case 'default':
      default:
        return cn(
          'bg-surface',
          bordered && 'border border-border/25'
        );
    }
  };

  const paddingClasses = {
    none: 'p-0',
    sm: 'p-[var(--card-padding-sm)]',
    md: 'p-[var(--card-padding-md)]',
    lg: 'p-[var(--card-padding-lg)]',
  };

  const shadowClasses = {
    none: '',
    sm: 'shadow-[var(--card-shadow-sm)]',
    md: 'shadow-[var(--card-shadow-md)]',
    lg: 'shadow-[var(--card-shadow-lg)]',
  };

  const cornerClasses = cutCorner === 'top-left'
    ? 'rounded-[var(--card-radius-default)] rounded-tl-[var(--card-radius-cut)]'
    : 'rounded-[var(--card-radius-default)] rounded-tr-[var(--card-radius-cut)]';

  return (
    <div
      className={cn(
        cornerClasses,
        'transition-all duration-normal',
        onClick && 'cursor-pointer hover:scale-[1.02]',
        paddingClasses[padding],
        shadowClasses[elevation],
        getVariantClasses(),
        className
      )}
      onClick={onClick}
    >
      {children}
    </div>
  );
};

export default Card;
