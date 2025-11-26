import React from 'react';
import { theme } from '@/theme.config';

interface SmartLogoProps {
  className?: string;
  width?: number | string;
  height?: number | string;
  alt?: string;
  variant?: 'primary' | 'dark' | 'small';
  preferSvg?: boolean; // Prefer SVG over optimized formats
}

/**
 * SmartLogo: Serves the best logo format based on browser support
 * Falls back: WebP -> PNG -> SVG
 *
 * Usage:
 * <SmartLogo width={200} height={60} />
 * <SmartLogo preferSvg /> // Force SVG for crisp scaling
 */
export const SmartLogo: React.FC<SmartLogoProps> = ({
  className = '',
  width = 200,
  height = 60,
  alt,
  preferSvg = false,
}) => {
  const logoAlt = alt || theme.branding.name;

  if (preferSvg) {
    // SVG format for perfect scaling at any size
    return (
      <img
        src="/assets/logos/logo.svg"
        alt={logoAlt}
        width={width}
        height={height}
        className={className}
        style={{ objectFit: 'contain' }}
      />
    );
  }

  // Use picture element for progressive enhancement
  // Browsers use the first format they support
  return (
    <picture>
      {/* WebP - Most efficient, ~30% smaller than PNG */}
      <source srcSet="/assets/logos/logo.webp" type="image/webp" />

      {/* PNG - Universal fallback */}
      <source srcSet="/assets/logos/logo.png" type="image/png" />

      {/* SVG - Final fallback for maximum compatibility */}
      <img
        src="/assets/logos/logo.svg"
        alt={logoAlt}
        width={width}
        height={height}
        className={className}
        style={{ objectFit: 'contain' }}
      />
    </picture>
  );
};

export default SmartLogo;
