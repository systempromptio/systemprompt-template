import React from 'react';

interface GradientBackgroundProps {
  children: React.ReactNode;
  className?: string;
}

/**
 * Gradient background using theme CSS variables
 * Radial gradients use --color-primary-rgb for dynamic theming
 */
export const GradientBackground: React.FC<GradientBackgroundProps> = ({
  children,
  className = '',
}) => {
  return (
    <div className={`relative h-full overflow-hidden ${className}`}>
      {/* Base gradient from theme */}
      <div
        className="fixed inset-0 z-0"
        style={{
          background: 'linear-gradient(180deg, var(--color-background-dark) 0%, var(--color-surface-dark) 70%, var(--color-background) 100%)',
        }}
      />

      {/* Top right radial gradient (Large) */}
      <div
        className="fixed z-0 animate-pulse-soft"
        style={{
          top: '-150px',
          right: '-150px',
          width: '600px',
          height: '600px',
          borderRadius: '300px',
          opacity: 1,
          background:
            'radial-gradient(circle, rgba(var(--color-primary-rgb), 0.35) 0%, rgba(var(--color-primary-rgb), 0.28) 20%, rgba(var(--color-primary-rgb), 0.20) 35%, rgba(var(--color-primary-rgb), 0.12) 50%, rgba(var(--color-primary-rgb), 0.06) 70%, rgba(var(--color-primary-rgb), 0.02) 85%, transparent 100%)',
        }}
      />

      {/* Left center radial gradient (Small) */}
      <div
        className="fixed z-0 animate-pulse-soft"
        style={{
          left: '-150px',
          top: '58%',
          marginTop: '-150px',
          width: '300px',
          height: '300px',
          borderRadius: '150px',
          opacity: 1,
          background:
            'radial-gradient(circle, rgba(var(--color-primary-rgb), 0.40) 0%, rgba(var(--color-primary-rgb), 0.32) 15%, rgba(var(--color-primary-rgb), 0.22) 30%, rgba(var(--color-primary-rgb), 0.14) 45%, rgba(var(--color-primary-rgb), 0.08) 60%, rgba(var(--color-primary-rgb), 0.04) 80%, transparent 100%)',
        }}
      />

      {/* Bottom right radial gradient (Large) */}
      <div
        className="fixed z-0 animate-pulse-soft"
        style={{
          bottom: '-200px',
          right: '-200px',
          width: '500px',
          height: '500px',
          borderRadius: '250px',
          opacity: 1,
          background:
            'radial-gradient(circle, rgba(var(--color-primary-rgb), 0.32) 0%, rgba(var(--color-primary-rgb), 0.26) 25%, rgba(var(--color-primary-rgb), 0.18) 40%, rgba(var(--color-primary-rgb), 0.12) 55%, rgba(var(--color-primary-rgb), 0.06) 70%, rgba(var(--color-primary-rgb), 0.03) 85%, transparent 100%)',
        }}
      />

      {/* Content */}
      <div className="relative z-10 h-full">{children}</div>
    </div>
  );
};

export default GradientBackground;
