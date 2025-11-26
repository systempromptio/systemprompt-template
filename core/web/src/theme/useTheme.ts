import { useContext } from 'react';
import { ThemeContext } from './ThemeProvider';
import type { ThemeContextType, AdaptedTheme } from './ThemeProvider';

/**
 * Hook to access theme context
 */
export const useTheme = (): ThemeContextType => {
  const context = useContext(ThemeContext);

  if (context === undefined) {
    throw new Error('useTheme must be used within a ThemeProvider');
  }

  return context;
};

/**
 * Hook to access theme values
 */
export const useThemeValues = (): AdaptedTheme => {
  const { theme } = useTheme();
  return theme;
};

export default useTheme;
