/**
 * SystemPrompt Theme Type Definitions
 * Auto-generated types for theme.config.ts
 * These types ensure type safety when working with theme configuration
 *
 * @module types/theme.types
 * @description Comprehensive type definitions for the theming system,
 *              including colors, typography, spacing, and layout configurations
 */

/**
 * Application branding configuration
 * @interface ThemeBranding
 * @property {string} name - Internal name of the application
 * @property {string} title - Public title displayed to users
 * @property {string} description - Meta description for SEO
 * @property {string} themeColor - Primary theme color (hex format)
 * @property {Object} logo - Logo configuration for different contexts
 * @property {string} logo.primary - Path to primary logo asset
 * @property {string} logo.dark - Path to dark mode logo asset
 * @property {string} logo.small - Path to small/favicon-sized logo
 * @property {string} favicon - Path to favicon file
 */
export interface ThemeBranding {
  name: string;
  title: string;
  description: string;
  themeColor: string;
  logo: {
    primary: string;
    dark: string;
    small: string;
  };
  favicon: string;
}

/**
 * Individual font file configuration
 * @interface FontFile
 * @property {string} path - Path to font file (relative to /assets/fonts)
 * @property {number} weight - CSS font-weight value (300-900)
 * @property {'normal' | 'italic' | 'oblique'} style - CSS font-style value
 */
export interface FontFile {
  path: string;
  weight: number;
  style: 'normal' | 'italic' | 'oblique';
}

/**
 * Complete font family definition with fallbacks
 * @interface FontDefinition
 * @property {string} family - Font family name used in CSS
 * @property {string} fallback - System font fallback chain (CSS variable)
 * @property {FontFile[]} files - Array of font files for different weights
 */
export interface FontDefinition {
  family: string;
  fallback: string;
  files: FontFile[];
}

/**
 * Application font configuration
 * @interface ThemeFonts
 * @property {FontDefinition} body - Font used for body text content
 * @property {FontDefinition} heading - Font used for headings (h1-h6)
 * @property {FontDefinition} brand - Font used for brand/logo text
 */
export interface ThemeFonts {
  body: FontDefinition;
  heading: FontDefinition;
  brand: FontDefinition;
}

/**
 * Color value representation in multiple formats
 * @interface ColorValue
 * @property {string} hsl - HSL color format (hsl(h, s%, l%))
 * @property {[number, number, number] | null} rgb - RGB tuple [r, g, b] or null if unavailable
 */
export interface ColorValue {
  hsl: string;
  rgb: [number, number, number] | null;
}

/**
 * Surface color variants for UI elements
 * @interface SurfaceColors
 * @property {string} default - Default surface color for cards and containers
 * @property {string} dark - Dark variant for dark mode
 * @property {string} variant - Alternate surface color for distinction
 * @property {string} secondaryContainer - Container for secondary information
 * @property {string} errorContainer - Container for error states
 */
export interface SurfaceColors {
  default: string;
  dark: string;
  variant: string;
  secondaryContainer: string;
  errorContainer: string;
}

/**
 * Text color semantic meanings
 * @interface TextColors
 * @property {string} primary - Main text color for readability
 * @property {string} secondary - Secondary text for less important content
 * @property {string} inverted - Inverted text color (opposite contrast)
 * @property {string} disabled - Color for disabled/inactive states
 */
export interface TextColors {
  primary: string;
  secondary: string;
  inverted: string;
  disabled: string;
}

/**
 * Background color variants
 * @interface BackgroundColors
 * @property {string} default - Default background color
 * @property {string} dark - Dark mode background color
 */
export interface BackgroundColors {
  default: string;
  dark: string;
}

/**
 * Border color variants
 * @interface BorderColors
 * @property {string} default - Standard border color
 * @property {string} dark - Dark mode border color
 * @property {string} outline - Prominent outline color for focus states
 */
export interface BorderColors {
  default: string;
  dark: string;
  outline: string;
}

/**
 * Complete color palette for a theme mode (light or dark)
 * @interface ColorPalette
 * @property {ColorValue} primary - Primary brand color
 * @property {ColorValue} secondary - Secondary accent color
 * @property {string} success - Color for success states
 * @property {string} warning - Color for warning states
 * @property {string} error - Color for error states
 * @property {SurfaceColors} surface - Surface color variants
 * @property {TextColors} text - Text color variants
 * @property {BackgroundColors} background - Background color variants
 * @property {BorderColors} border - Border color variants
 */
export interface ColorPalette {
  primary: ColorValue;
  secondary: ColorValue;
  success: string;
  warning: string;
  error: string;
  surface: SurfaceColors;
  text: TextColors;
  background: BackgroundColors;
  border: BorderColors;
}

/**
 * Theme colors for light and dark modes
 * @interface ThemeColors
 * @property {ColorPalette} light - Color palette for light mode
 * @property {ColorPalette} dark - Color palette for dark mode
 */
export interface ThemeColors {
  light: ColorPalette;
  dark: ColorPalette;
}

/**
 * Typography font sizes scale
 * @interface TypographySizes
 * @property {string} xs - Extra small (12px) for captions
 * @property {string} sm - Small (14px) for labels
 * @property {string} md - Medium (15px) for normal text
 * @property {string} lg - Large (18px) for larger text
 * @property {string} xl - Extra large (20px) for subheadings
 * @property {string} xxl - Extra extra large (28px) for main headings
 */
export interface TypographySizes {
  xs: string;
  sm: string;
  md: string;
  lg: string;
  xl: string;
  xxl: string;
}

/**
 * Typography font weight values
 * @interface TypographyWeights
 * @property {number} regular - Normal font weight (400)
 * @property {number} medium - Medium font weight (500)
 * @property {number} semibold - Semi-bold font weight (600)
 * @property {number} bold - Bold font weight (700)
 */
export interface TypographyWeights {
  regular: number;
  medium: number;
  semibold: number;
  bold: number;
}

/**
 * Complete typography configuration
 * @interface ThemeTypography
 * @property {TypographySizes} sizes - Font size scale values
 * @property {TypographyWeights} weights - Font weight values
 */
export interface ThemeTypography {
  sizes: TypographySizes;
  weights: TypographyWeights;
}

/**
 * Spacing scale for margins, paddings, and gaps
 * Follows a consistent 4px base unit progression
 * @interface SpacingScale
 * @property {string} xs - Extra small (4px)
 * @property {string} sm - Small (8px)
 * @property {string} md - Medium (16px)
 * @property {string} lg - Large (24px)
 * @property {string} xl - Extra large (32px)
 * @property {string} xxl - Extra extra large (48px)
 */
export interface SpacingScale {
  xs: string;
  sm: string;
  md: string;
  lg: string;
  xl: string;
  xxl: string;
}

/**
 * Border radius scale for rounded corners
 * @interface RadiusScale
 * @property {string} xs - Extra small radius (2px)
 * @property {string} sm - Small radius (4px)
 * @property {string} md - Medium radius (8px)
 * @property {string} lg - Large radius (12px)
 * @property {string} xl - Extra large radius (16px)
 * @property {string} xxl - Extra extra large radius (24px)
 * @property {string} round - Fully rounded (9999px) for pills/circles
 */
export interface RadiusScale {
  xs: string;
  sm: string;
  md: string;
  lg: string;
  xl: string;
  xxl: string;
  round: string;
}

/**
 * Box shadow scale for elevation effects
 * @interface ShadowScale
 * @property {string} sm - Small subtle shadow
 * @property {string} md - Medium elevation shadow
 * @property {string} lg - Large prominent shadow
 * @property {string} accent - Accent-colored shadow for special elements
 */
export interface ShadowScale {
  sm: string;
  md: string;
  lg: string;
  accent: string;
}

/**
 * Shadows for light and dark modes
 * @interface ThemeShadows
 * @property {ShadowScale} light - Shadows for light mode
 * @property {ShadowScale} dark - Shadows for dark mode
 */
export interface ThemeShadows {
  light: ShadowScale;
  dark: ShadowScale;
}

/**
 * Animation transition timing durations
 * @interface ThemeAnimation
 * @property {string} fast - Fast animations (200ms) for quick feedback
 * @property {string} normal - Normal animations (300ms) for standard transitions
 * @property {string} slow - Slow animations (500ms) for dramatic effects
 */
export interface ThemeAnimation {
  fast: string;
  normal: string;
  slow: string;
}

/**
 * Z-index layering for stacking context management
 * @interface ThemeZIndex
 * @property {number} base - Base layer (0) for normal content
 * @property {number} content - Content layer (10) for elevated elements
 * @property {number} navigation - Navigation layer (100) for headers/sidebars
 * @property {number} modal - Modal layer (1000) for dialogs/overlays
 * @property {number} tooltip - Tooltip layer (10000) for tooltips and popovers
 */
export interface ThemeZIndex {
  base: number;
  content: number;
  navigation: number;
  modal: number;
  tooltip: number;
}

/**
 * Sidebar dimension configuration
 * @interface SidebarConfig
 * @property {string} width - Default width percentage
 * @property {string} minWidth - Minimum width in pixels
 * @property {string} maxWidth - Maximum width in pixels
 */
export interface SidebarConfig {
  width: string;
  minWidth: string;
  maxWidth: string;
}

/**
 * Application layout dimensions
 * @interface ThemeLayout
 * @property {string} headerHeight - Height of main header/navigation bar
 * @property {SidebarConfig} sidebarLeft - Left sidebar configuration
 * @property {SidebarConfig} sidebarRight - Right sidebar configuration
 * @property {string} navHeight - Height of secondary navigation
 * @property {string} contentMaxWidth - Maximum width for content area
 */
export interface ThemeLayout {
  headerHeight: string;
  sidebarLeft: SidebarConfig;
  sidebarRight: SidebarConfig;
  navHeight: string;
  contentMaxWidth: string;
}

/**
 * Card component border radius variants
 * @interface CardRadius
 * @property {string} default - Standard card corner radius
 * @property {string} cut - Cut/chamfered corner style
 */
export interface CardRadius {
  default: string;
  cut: string;
}

/**
 * Card component padding values
 * @interface CardPadding
 * @property {string} sm - Small padding for compact cards
 * @property {string} md - Medium padding for standard cards
 * @property {string} lg - Large padding for spacious cards
 */
export interface CardPadding {
  sm: string;
  md: string;
  lg: string;
}

/**
 * Card gradient overlay configuration
 * @interface CardGradient
 * @property {string} start - Gradient start color
 * @property {string} mid - Gradient middle color
 * @property {string} end - Gradient end color
 */
export interface CardGradient {
  start: string;
  mid: string;
  end: string;
}

/**
 * Complete card styling configuration
 * @interface ThemeCard
 * @property {CardRadius} radius - Border radius variants
 * @property {CardPadding} padding - Padding scale
 * @property {CardGradient} gradient - Gradient overlay configuration
 */
export interface ThemeCard {
  radius: CardRadius;
  padding: CardPadding;
  gradient: CardGradient;
}

/**
 * Mobile-specific typography configuration
 * @interface MobileTypography
 * @property {TypographySizes} sizes - Mobile font sizes (may differ from desktop)
 */
export interface MobileTypography {
  sizes: TypographySizes;
}

/**
 * Mobile-specific layout dimensions
 * @interface MobileLayout
 * @property {string} headerHeight - Mobile header height (typically smaller than desktop)
 * @property {string} navHeight - Mobile navigation bar height
 */
export interface MobileLayout {
  headerHeight: string;
  navHeight: string;
}

/**
 * Mobile-specific card styling
 * @interface MobileCard
 * @property {CardPadding} padding - Card padding for mobile screens
 */
export interface MobileCard {
  padding: CardPadding;
}

/**
 * Complete mobile-specific theme configuration
 * @interface ThemeMobile
 * @property {SpacingScale} spacing - Mobile spacing scale (may differ from desktop)
 * @property {MobileTypography} typography - Mobile typography overrides
 * @property {MobileLayout} layout - Mobile layout dimensions
 * @property {MobileCard} card - Mobile card styling
 */
export interface ThemeMobile {
  spacing: SpacingScale;
  typography: MobileTypography;
  layout: MobileLayout;
  card: MobileCard;
}

/**
 * Minimum touch target sizes for accessibility
 * Follows Material Design guidelines (48x48px minimum)
 * @interface TouchTargets
 * @property {string} default - Standard touch target (44px)
 * @property {string} sm - Small touch target (40px)
 * @property {string} lg - Large touch target (48px)
 */
export interface TouchTargets {
  default: string;
  sm: string;
  lg: string;
}

/**
 * Complete theme configuration
 * The root interface containing all theme properties for both light and dark modes
 * @interface Theme
 * @property {ThemeBranding} branding - Brand identity configuration
 * @property {ThemeFonts} fonts - Font families and weights
 * @property {ThemeColors} colors - Color palettes for light and dark modes
 * @property {ThemeTypography} typography - Typography scale (sizes and weights)
 * @property {SpacingScale} spacing - Spacing scale for all margins and paddings
 * @property {RadiusScale} radius - Border radius scale
 * @property {ThemeShadows} shadows - Shadow elevation system
 * @property {ThemeAnimation} animation - Animation timing durations
 * @property {ThemeZIndex} zIndex - Z-index stacking values
 * @property {ThemeLayout} layout - Desktop layout dimensions
 * @property {ThemeCard} card - Card component styling
 * @property {ThemeMobile} mobile - Mobile-specific overrides
 * @property {TouchTargets} touchTargets - Minimum touch target sizes
 */
export interface Theme {
  branding: ThemeBranding;
  fonts: ThemeFonts;
  colors: ThemeColors;
  typography: ThemeTypography;
  spacing: SpacingScale;
  radius: RadiusScale;
  shadows: ThemeShadows;
  animation: ThemeAnimation;
  zIndex: ThemeZIndex;
  layout: ThemeLayout;
  card: ThemeCard;
  mobile: ThemeMobile;
  touchTargets: TouchTargets;
}
