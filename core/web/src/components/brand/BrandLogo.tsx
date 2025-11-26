import React from 'react';
import { theme } from '@/theme.config';

interface BrandLogoProps {
  className?: string;
  width?: number | string;
  height?: number | string;
  alt?: string;
  variant?: 'primary' | 'dark' | 'small';
  format?: 'svg' | 'webp' | 'png'; // Explicitly choose format
}

/**
 * BrandLogo: Displays brand logo from config
 * Supports SVG (scalable), WebP (optimized), and PNG (universal)
 * Defaults to SVG for crisp rendering at any size
 */
export const BrandLogo: React.FC<BrandLogoProps> = ({
  className = '',
  width = 200,
  height = 60,
  alt,
  variant = 'primary',
  format = 'svg',
}) => {
  const logoAlt = alt || theme.branding.name;

  // Get logo source from config based on variant and format
  const getLogoSrc = (): string => {
    const variantConfig = theme.branding.logo[variant] as any;

    // If variant has the requested format, use it
    if (variantConfig && variantConfig[format]) {
      return variantConfig[format];
    }

    // Fallback: use png if format not available
    if (variantConfig && variantConfig.png) {
      return variantConfig.png;
    }

    // Ultimate fallback: use primary png
    return theme.branding.logo.primary.png;
  };

  const logoSrc = getLogoSrc();

  return (
    <img
      src={logoSrc}
      alt={logoAlt}
      width={width}
      height={height}
      className={className}
      style={{ objectFit: 'contain' }}
    />
  );
};

export default BrandLogo;
