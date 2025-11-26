import React, { createContext, useMemo, useEffect } from 'react';
import { theme as generatedTheme } from '@/theme.config';

// Adapter type to maintain compatibility with existing components
export interface AdaptedTheme {
  colors: {
    primary: string;
    secondary: string;
    success: string;
    warning: string;
    error: string;
    surface: string;
    surfaceDark: string;
    surfaceVariant: string;
    secondaryContainer: string;
    errorContainer: string;
    textPrimary: string;
    textSecondary: string;
    textInverted: string;
    textDisabled: string;
    onSurface: string;
    onSurfaceVariant: string;
    onPrimary: string;
    onSecondary: string;
    onBackground: string;
    onError: string;
    background: string;
    backgroundDark: string;
    border: string;
    borderDark: string;
    outline: string;
    shadow: string;
  };
  spacing: typeof generatedTheme.spacing;
  typography: typeof generatedTheme.typography;
  borderRadius: typeof generatedTheme.radius;
  shadows: typeof generatedTheme.shadows;
  animation: typeof generatedTheme.animation;
  zIndex: typeof generatedTheme.zIndex;
  components: {
    card: {
      borderRadius: {
        default: number;
        topRight: number;
      };
      padding: typeof generatedTheme.card.padding;
      gradient: typeof generatedTheme.card.gradient;
    };
  };
  layout: typeof generatedTheme.layout;
  dark: boolean;
}

export interface ThemeContextType {
  theme: AdaptedTheme;
}

// Adapt generated theme to old structure for compatibility
const adaptTheme = (isDark: boolean): AdaptedTheme => {
  const colors = isDark ? generatedTheme.colors.dark : generatedTheme.colors.light;

  return {
    colors: {
      primary: typeof colors.primary === 'object' ? colors.primary.hsl : colors.primary,
      secondary: typeof colors.secondary === 'object' ? colors.secondary.hsl : colors.secondary,
      success: colors.success,
      warning: colors.warning,
      error: colors.error,
      surface: colors.surface.default,
      surfaceDark: colors.surface.dark,
      surfaceVariant: colors.surface.variant,
      secondaryContainer: colors.surface.secondaryContainer,
      errorContainer: colors.surface.errorContainer,
      textPrimary: colors.text.primary,
      textSecondary: colors.text.secondary,
      textInverted: colors.text.inverted,
      textDisabled: colors.text.disabled,
      onSurface: colors.text.primary,
      onSurfaceVariant: colors.text.secondary,
      onPrimary: colors.text.inverted,
      onSecondary: colors.text.primary,
      onBackground: colors.text.primary,
      onError: colors.text.inverted,
      background: colors.background.default,
      backgroundDark: colors.background.dark,
      border: colors.border.default,
      borderDark: colors.border.dark,
      outline: colors.border.outline,
      shadow: 'rgba(0, 0, 0, 0.3)',
    },
    spacing: generatedTheme.spacing,
    typography: generatedTheme.typography,
    borderRadius: generatedTheme.radius,
    shadows: generatedTheme.shadows,
    animation: generatedTheme.animation,
    zIndex: generatedTheme.zIndex,
    components: {
      card: {
        borderRadius: {
          default: parseInt(generatedTheme.card.radius.default),
          topRight: parseInt(generatedTheme.card.radius.cut),
        },
        padding: generatedTheme.card.padding,
        gradient: generatedTheme.card.gradient,
      },
    },
    layout: generatedTheme.layout,
    dark: isDark,
  };
};

export const ThemeContext = createContext<ThemeContextType>({
  theme: adaptTheme(true),
});

export const ThemeProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const theme = useMemo(() => adaptTheme(true), []);

  useEffect(() => {
    document.documentElement.classList.add('dark');
  }, []);

  const contextValue = useMemo(
    () => ({
      theme,
    }),
    [theme]
  );

  return <ThemeContext.Provider value={contextValue}>{children}</ThemeContext.Provider>;
};

export default ThemeProvider;
