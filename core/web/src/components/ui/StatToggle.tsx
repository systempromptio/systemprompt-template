import React from 'react';
import type { LucideIcon } from 'lucide-react';
import { cn } from '@/lib/utils/cn';
import { useThemeValues } from '@/theme';

interface StatToggleProps {
  label: string;
  count: number;
  icon?: LucideIcon;
  isSelected?: boolean;
  disabled?: boolean;
  onClick?: () => void;
}

export const StatToggle: React.FC<StatToggleProps> = ({
  label,
  count,
  icon: Icon,
  isSelected = false,
  disabled = false,
  onClick,
}) => {
  const theme = useThemeValues();

  const getBackgroundStyle = () => {
    if (disabled) {
      return {
        background: 'transparent',
      };
    }
    if (isSelected) {
      const { card } = theme.components;
      return {
        background: `linear-gradient(135deg, ${card.gradient.start}, ${card.gradient.mid}, ${card.gradient.end})`,
      };
    }
    return {
      background: 'transparent',
    };
  };

  return (
    <button
      onClick={disabled ? undefined : onClick}
      disabled={disabled}
      className={cn(
        'group relative flex items-center gap-xs px-md py-sm',
        'border transition-all duration-fast',
        'focus:outline-none',
        disabled
          ? 'border-text-disabled/30 opacity-40 cursor-not-allowed grayscale'
          : cn(
              'focus:ring-2 focus:ring-primary focus:ring-offset-2',
              isSelected
                ? 'border-primary text-white'
                : 'border-primary/30 text-text-secondary hover:border-primary/60 hover:scale-105'
            )
      )}
      style={{
        borderRadius: `${theme.components.card.borderRadius.default}px`,
        borderTopRightRadius: `${theme.components.card.borderRadius.topRight}px`,
        ...getBackgroundStyle(),
      }}
    >
      {Icon && (
        <Icon
          className={cn(
            'w-4 h-4',
            disabled
              ? 'text-text-disabled'
              : isSelected
                ? 'text-white'
                : 'text-primary'
          )}
        />
      )}

      <span className={cn(
        'text-sm font-heading font-medium uppercase tracking-wide',
        disabled
          ? 'text-text-disabled'
          : isSelected
            ? 'text-white'
            : 'text-text-primary'
      )}>
        {label}
      </span>

      <span
        className={cn(
          'flex items-center justify-center min-w-[24px] h-5 px-xs',
          'rounded-full text-xs font-body font-semibold',
          disabled
            ? 'bg-text-disabled/10 text-text-disabled'
            : isSelected
              ? 'bg-white/20 text-white'
              : 'bg-primary/10 text-primary'
        )}
      >
        {count}
      </span>
    </button>
  );
};

export default StatToggle;
