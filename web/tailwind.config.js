/**
 * Tailwind CSS Configuration
 *
 * Customizes Tailwind CSS for the SystemPrompt application with:
 * - Theme-driven design tokens (colors, spacing, typography)
 * - Dark mode support via CSS class toggle
 * - Responsive utilities for mobile-first design
 * - Custom animations and visual effects
 * - Semantic naming for design consistency
 *
 * Content Scanning:
 * - Scans HTML, TS, TSX, JS, JSX files for Tailwind class usage
 * - Removes unused styles in production (CSS purging)
 * - Generates exact CSS needed for application
 *
 * Dark Mode:
 * - Strategy: 'class' - toggle with HTML classList
 * - Usage: Add 'dark' class to html element to enable dark theme
 * - JavaScript: useTheme() hook handles toggling
 *
 * Screens (Responsive Breakpoints):
 * - sm: 640px - Small devices (large phones)
 * - md: 768px - Tablets
 * - lg: 1024px - Desktops
 * - xl: 1280px - Large desktops
 * - 2xl: 1440px - Ultra-wide screens
 *
 * Theme Extends:
 * - Colors: Using CSS variables from --color-* custom properties
 * - Spacing: 4px-based scale (xs=4px, sm=8px, md=16px, lg=24px, xl=32px, xxl=48px)
 * - Typography: Custom font families and sizes
 * - Animations: Entrance effects, transitions, state changes
 *
 * Key Customizations:
 * - Layout utilities: sidebar widths, header heights, grid templates
 * - Touch targets: Minimum 40-48px for mobile accessibility
 * - Animations: Smooth transitions for state changes and interactions
 * - Z-index: Explicit layering for modals, tooltips, navigation
 *
 * CSS Variables Integration:
 * - All colors defined as CSS custom properties
 * - Allows dynamic theme switching without CSS rebuild
 * - Fallbacks to defaults if variables unavailable
 *
 * @type {import('tailwindcss').Config}
 * @see https://tailwindcss.com/docs/configuration
 * @see {@link ./src/styles/variables.css} CSS custom properties
 * @see {@link ./theme.config.ts} Theme token definitions
 */

/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  darkMode: 'class',
  plugins: [
    require('@tailwindcss/typography'),
  ],
  theme: {
    screens: {
      'sm': '640px',
      'md': '768px',
      'lg': '1024px',
      'xl': '1280px',
      '2xl': '1440px',
    },
    extend: {
      colors: {
        primary: 'var(--color-primary)',
        'primary-10': 'var(--color-border-primary-10)',
        'primary-15': 'var(--color-border-primary-15)',
        'primary-20': 'var(--color-border-primary-20)',
        secondary: 'var(--color-secondary)',
        success: 'var(--color-success)',
        warning: 'var(--color-warning)',
        error: 'var(--color-error)',

        surface: {
          DEFAULT: 'var(--color-surface)',
          dark: 'var(--color-surface-dark)',
          variant: 'var(--color-surface-variant)',
        },

        text: {
          primary: 'var(--color-text-primary)',
          secondary: 'var(--color-text-secondary)',
          inverted: 'var(--color-text-inverted)',
          disabled: 'var(--color-text-disabled)',
        },

        background: {
          DEFAULT: 'var(--color-background)',
          dark: 'var(--color-background-dark)',
        },

        border: {
          DEFAULT: 'var(--color-border)',
          dark: 'var(--color-border-dark)',
        },

        outline: 'var(--color-outline)',
      },

      borderColor: {
        'primary-10': 'var(--color-border-primary-10)',
        'primary-15': 'var(--color-border-primary-15)',
        'primary-20': 'var(--color-border-primary-20)',
      },

      spacing: {
        xs: 'var(--spacing-xs)',
        sm: 'var(--spacing-sm)',
        md: 'var(--spacing-md)',
        lg: 'var(--spacing-lg)',
        xl: 'var(--spacing-xl)',
        xxl: 'var(--spacing-xxl)',
      },

      fontFamily: {
        body: 'var(--font-body)',
        heading: 'var(--font-heading)',
        brand: 'var(--font-brand)',
      },

      fontSize: {
        xs: 'var(--font-size-xs)',
        sm: 'var(--font-size-sm)',
        md: 'var(--font-size-md)',
        lg: 'var(--font-size-lg)',
        xl: 'var(--font-size-xl)',
        xxl: 'var(--font-size-xxl)',
      },

      fontWeight: {
        regular: 'var(--font-weight-regular)',
        medium: 'var(--font-weight-medium)',
        semibold: 'var(--font-weight-semibold)',
        bold: 'var(--font-weight-bold)',
      },

      borderRadius: {
        xs: 'var(--radius-xs)',
        sm: 'var(--radius-sm)',
        md: 'var(--radius-md)',
        lg: 'var(--radius-lg)',
        xl: 'var(--radius-xl)',
        xxl: 'var(--radius-xxl)',
        round: 'var(--radius-round)',
      },

      boxShadow: {
        sm: 'var(--shadow-sm)',
        md: 'var(--shadow-md)',
        lg: 'var(--shadow-lg)',
        accent: 'var(--shadow-accent)',
      },

      transitionDuration: {
        fast: 'var(--animation-fast)',
        normal: 'var(--animation-normal)',
        slow: 'var(--animation-slow)',
      },

      zIndex: {
        base: 'var(--z-base)',
        content: 'var(--z-content)',
        navigation: 'var(--z-navigation)',
        modal: 'var(--z-modal)',
        tooltip: 'var(--z-tooltip)',
      },

      width: {
        'sidebar-left': '20%',
        'sidebar-right': '15%',
      },

      minWidth: {
        'sidebar-left': '240px',
        'sidebar-right': '200px',
        'touch': '44px',
        'touch-sm': '40px',
        'touch-lg': '48px',
      },

      minHeight: {
        'touch': '44px',
        'touch-sm': '40px',
        'touch-lg': '48px',
      },

      maxWidth: {
        'sidebar-left': '320px',
        'sidebar-right': '280px',
        'content': '1600px',
      },

      height: {
        'header': '48px',
      },

      gridTemplateColumns: {
        'layout': 'minmax(240px, 20%) 1fr minmax(200px, 15%)',
        'layout-no-right': 'minmax(240px, 20%) 1fr',
        'layout-mobile': '1fr',
      },

      gridTemplateRows: {
        'layout': '48px 1fr',
      },

      keyframes: {
        slideInUp: {
          from: { opacity: '0', transform: 'translateY(20px)' },
          to: { opacity: '1', transform: 'translateY(0)' },
        },
        slideInRight: {
          from: { opacity: '0', transform: 'translateX(-20px)' },
          to: { opacity: '1', transform: 'translateX(0)' },
        },
        scaleIn: {
          from: { opacity: '0', transform: 'scale(0.9)' },
          to: { opacity: '1', transform: 'scale(1)' },
        },
        expandIn: {
          from: { opacity: '0', maxHeight: '0' },
          to: { opacity: '1', maxHeight: '500px' },
        },
        fadeIn: {
          from: { opacity: '0' },
          to: { opacity: '1' },
        },
        pulse: {
          '0%, 100%': { opacity: '1' },
          '50%': { opacity: '0.5' },
        },
        shimmer: {
          '0%': { backgroundPosition: '-1000px 0' },
          '100%': { backgroundPosition: '1000px 0' },
        },
        progressIndeterminate: {
          '0%': { transform: 'translateX(-100%)' },
          '100%': { transform: 'translateX(100%)' },
        },
        slideUp: {
          '0%': { transform: 'translateY(100%)', opacity: '0' },
          '100%': { transform: 'translateY(0)', opacity: '1' },
        },
        pulseGlow: {
          '0%, 100%': {
            filter: 'brightness(1) saturate(1)',
            transform: 'scale(1)',
          },
          '50%': {
            filter: 'brightness(1.1) saturate(1.2)',
            transform: 'scale(1.02)',
          },
        },
        shimmerSweep: {
          '0%': {
            backgroundPosition: '-200% 0',
          },
          '100%': {
            backgroundPosition: '200% 0',
          },
        },
      },

      animation: {
        slideInUp: 'slideInUp 0.3s ease-out forwards',
        slideInRight: 'slideInRight 0.3s ease-out forwards',
        scaleIn: 'scaleIn 0.2s ease-out forwards',
        expandIn: 'expandIn 0.4s ease-in-out forwards',
        fadeIn: 'fadeIn 0.2s ease-in forwards',
        pulse: 'pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite',
        shimmer: 'shimmer 2s linear infinite',
        progressIndeterminate: 'progressIndeterminate 1.5s linear infinite',
        slideUp: 'slideUp 0.3s ease-out',
        pulseGlow: 'pulseGlow 3s ease-in-out infinite',
        shimmerSweep: 'shimmerSweep 3s linear infinite',
      },
    },
  },
}
