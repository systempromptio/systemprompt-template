import React from 'react';
import { useThemeValues } from '@/theme';
import { theme as themeConfig } from '@/theme.config';

export interface BrandTextProps extends React.HTMLAttributes<HTMLSpanElement> {
  text?: string;
  size?: 'small' | 'medium' | 'large' | 'xlarge';
  inverted?: boolean;
  color?: string;
  children?: React.ReactNode;
}

export const BrandText: React.FC<BrandTextProps> = ({
  text = themeConfig.branding.name,
  size = 'medium',
  inverted = false,
  color,
  className = '',
  children,
  ...rest
}) => {
  const theme = useThemeValues();

  const getFontSize = () => {
    const sizeMap = {
      small: themeConfig.typography.sizes.lg,    // 18px
      medium: themeConfig.typography.sizes.xl,   // 20px -> using xl for closer to 24px
      large: themeConfig.typography.sizes.xxl,   // 28px -> closest to 32px
      xlarge: '48px', // Keep custom size as it's larger than theme provides
    };
    return sizeMap[size];
  };

  const getTextColor = () => {
    if (color) return color;
    return inverted ? theme.colors.textInverted : theme.colors.primary;
  };

  return (
    <span
      className={`font-brand ${className}`}
      style={{
        fontSize: getFontSize(),
        color: getTextColor(),
        letterSpacing: '0.5px',
      }}
      {...rest}
    >
      {children || text}
    </span>
  );
};

export default BrandText;
