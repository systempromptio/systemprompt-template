#!/usr/bin/env node

import { readFileSync, writeFileSync, mkdirSync } from 'fs';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';
import yaml from 'js-yaml';
import Ajv from 'ajv';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const YAML_PATH = process.env.SYSTEMPROMPT_WEB_CONFIG_PATH || join(__dirname, '../../../crates/services/web/config.yml');
const METADATA_PATH = join(__dirname, '../../../crates/services/web/metadata.yml');
const SCHEMA_PATH = join(__dirname, 'theme-schema.json');
const CSS_OUTPUT = join(__dirname, '../src/styles/theme.generated.css');
const TS_OUTPUT = join(__dirname, '../src/theme.config.ts');

function generateCSSVariables(theme) {
  const vars = [];

  const colors = theme.colors.light;

  vars.push(`  --color-primary: ${colors.primary.hsl};`);
  vars.push(`  --color-primary-rgb: ${colors.primary.rgb.join(', ')};`);
  vars.push(`  --color-secondary: ${colors.secondary.hsl};`);
  vars.push(`  --color-success: ${colors.success};`);
  vars.push(`  --color-warning: ${colors.warning};`);
  vars.push(`  --color-error: ${colors.error};`);
  vars.push('');

  vars.push(`  --color-surface: ${colors.surface.default};`);
  vars.push(`  --color-surface-dark: ${colors.surface.dark};`);
  vars.push(`  --color-surface-variant: ${colors.surface.variant};`);
  vars.push(`  --color-secondary-container: ${colors.surface.secondaryContainer};`);
  vars.push(`  --color-error-container: ${colors.surface.errorContainer};`);
  vars.push('');

  vars.push(`  --color-text-primary: ${colors.text.primary};`);
  vars.push(`  --color-text-secondary: ${colors.text.secondary};`);
  vars.push(`  --color-text-inverted: ${colors.text.inverted};`);
  vars.push(`  --color-text-disabled: ${colors.text.disabled};`);
  vars.push('');

  vars.push(`  --color-background: ${colors.background.default};`);
  vars.push(`  --color-background-dark: ${colors.background.dark};`);
  vars.push('');

  vars.push(`  --color-border: ${colors.border.default};`);
  vars.push(`  --color-border-dark: ${colors.border.dark};`);
  vars.push(`  --color-outline: ${colors.border.outline};`);
  vars.push('');

  vars.push(`  --color-border-primary-10: rgba(var(--color-primary-rgb), 0.1);`);
  vars.push(`  --color-border-primary-15: rgba(var(--color-primary-rgb), 0.15);`);
  vars.push(`  --color-border-primary-20: rgba(var(--color-primary-rgb), 0.2);`);
  vars.push('');

  Object.entries(theme.spacing).forEach(([key, value]) => {
    vars.push(`  --spacing-${key}: ${value};`);
  });
  vars.push('');

  const fonts = theme.fonts;
  vars.push(`  --font-body: '${fonts.body.family}', ${fonts.body.fallback};`);
  vars.push(`  --font-heading: '${fonts.heading.family}', ${fonts.heading.fallback};`);
  vars.push(`  --font-brand: '${fonts.brand.family}', ${fonts.brand.fallback};`);
  vars.push('');

  Object.entries(theme.typography.sizes).forEach(([key, value]) => {
    vars.push(`  --font-size-${key}: ${value};`);
  });
  vars.push('');

  Object.entries(theme.typography.weights).forEach(([key, value]) => {
    vars.push(`  --font-weight-${key}: ${value};`);
  });
  vars.push('');

  Object.entries(theme.radius).forEach(([key, value]) => {
    vars.push(`  --radius-${key}: ${value};`);
  });
  vars.push('');

  Object.entries(theme.shadows.light).forEach(([key, value]) => {
    vars.push(`  --shadow-${key}: ${value};`);
  });
  vars.push('');

  Object.entries(theme.animation).forEach(([key, value]) => {
    vars.push(`  --animation-${key}: ${value};`);
  });
  vars.push('');

  Object.entries(theme.zIndex).forEach(([key, value]) => {
    vars.push(`  --z-${key}: ${value};`);
  });
  vars.push('');

  vars.push(`  --header-height: ${theme.layout.headerHeight};`);
  vars.push(`  --sidebar-left-width: ${theme.layout.sidebarLeft.width};`);
  vars.push(`  --sidebar-right-width: ${theme.layout.sidebarRight.width};`);
  vars.push(`  --nav-height: ${theme.layout.navHeight};`);
  vars.push('');

  vars.push(`  --card-radius-default: ${theme.card.radius.default};`);
  vars.push(`  --card-radius-cut: ${theme.card.radius.cut};`);
  vars.push(`  --card-padding-sm: ${theme.card.padding.sm};`);
  vars.push(`  --card-padding-md: ${theme.card.padding.md};`);
  vars.push(`  --card-padding-lg: ${theme.card.padding.lg};`);
  vars.push(`  --card-gradient-start: ${theme.card.gradient.start};`);
  vars.push(`  --card-gradient-mid: ${theme.card.gradient.mid};`);
  vars.push(`  --card-gradient-end: ${theme.card.gradient.end};`);
  vars.push(`  --card-shadow-sm: ${theme.shadows.light.sm};`);
  vars.push(`  --card-shadow-md: ${theme.shadows.light.md};`);
  vars.push(`  --card-shadow-lg: ${theme.shadows.light.lg};`);

  return vars.join('\n');
}

function generateDarkModeOverrides(theme) {
  const vars = [];
  const colors = theme.colors.dark;

  vars.push(`  --color-primary: ${colors.primary.hsl};`);
  vars.push(`  --color-secondary: ${colors.secondary.hsl};`);
  vars.push('');

  vars.push(`  --color-surface: ${colors.surface.default};`);
  vars.push(`  --color-surface-dark: ${colors.surface.dark};`);
  vars.push(`  --color-surface-variant: ${colors.surface.variant};`);
  vars.push(`  --color-secondary-container: ${colors.surface.secondaryContainer};`);
  vars.push(`  --color-error-container: ${colors.surface.errorContainer};`);
  vars.push('');

  vars.push(`  --color-text-primary: ${colors.text.primary};`);
  vars.push(`  --color-text-secondary: ${colors.text.secondary};`);
  vars.push(`  --color-text-inverted: ${colors.text.inverted};`);
  vars.push(`  --color-text-disabled: ${colors.text.disabled};`);
  vars.push('');

  vars.push(`  --color-background: ${colors.background.default};`);
  vars.push(`  --color-background-dark: ${colors.background.dark};`);
  vars.push('');

  vars.push(`  --color-border: ${colors.border.default};`);
  vars.push(`  --color-border-dark: ${colors.border.dark};`);
  vars.push(`  --color-outline: ${colors.border.outline};`);
  vars.push('');

  Object.entries(theme.shadows.dark).forEach(([key, value]) => {
    vars.push(`  --shadow-${key}: ${value};`);
  });

  return vars.join('\n');
}

function generateFontFaces(theme) {
  const fontFaces = [];

  Object.values(theme.fonts).forEach(font => {
    font.files.forEach(file => {
      fontFaces.push(`@font-face {
  font-family: '${font.family}';
  src: url('${file.path}') format('truetype');
  font-weight: ${file.weight};
  font-style: ${file.style};
  font-display: swap;
}`);
    });
  });

  return fontFaces.join('\n\n');
}

function generateMobileOverrides(theme) {
  const vars = [];

  Object.entries(theme.mobile.spacing).forEach(([key, value]) => {
    vars.push(`    --spacing-${key}: ${value};`);
  });
  vars.push('');

  Object.entries(theme.mobile.typography.sizes).forEach(([key, value]) => {
    vars.push(`    --font-size-${key}: ${value};`);
  });
  vars.push('');

  vars.push(`    --header-height: ${theme.mobile.layout.headerHeight};`);
  vars.push(`    --nav-height: ${theme.mobile.layout.navHeight};`);
  vars.push('');

  vars.push(`    --card-padding-sm: ${theme.mobile.card.padding.sm};`);
  vars.push(`    --card-padding-md: ${theme.mobile.card.padding.md};`);
  vars.push(`    --card-padding-lg: ${theme.mobile.card.padding.lg};`);

  return vars.join('\n');
}

function generateCSS(theme) {
  return `/**
 * GENERATED FILE - DO NOT EDIT MANUALLY
 * Generated from: crates/services/web/config.yml
 * Run 'npm run theme:generate' to regenerate
 */

@import "tailwindcss";

@theme {
${generateCSSVariables(theme)}
}

/* Font Face Declarations */
${generateFontFaces(theme)}

/* Dark Theme Overrides */
.dark {
${generateDarkModeOverrides(theme)}
}

/* Keyframe Animations */
@keyframes slideUp {
  0% { transform: translateY(100%); opacity: 0; }
  100% { transform: translateY(0); opacity: 1; }
}

@keyframes fadeIn {
  0% { opacity: 0; }
  100% { opacity: 1; }
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.5; }
}

@keyframes pulse-soft {
  0%, 100% { opacity: 0.9; transform: scale(1); }
  50% { opacity: 0.7; transform: scale(1.05); }
}

/* Base Styles */
* {
  border-color: var(--color-border);
}

html, body {
  margin: 0;
  padding: 0;
  width: 100%;
  height: 100%;
  overflow: hidden;
}

body {
  font-family: var(--font-body);
  background-color: var(--color-background);
  color: var(--color-text-primary);
  font-feature-settings: "rlig" 1, "calt" 1;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  font-size: var(--font-size-md);
}

#root {
  width: 100%;
  height: 100%;
  overflow: hidden;
}

* {
  font-family: var(--font-body);
}

h1, h2, h3, h4, h5, h6 {
  font-family: var(--font-heading);
}

code, pre {
  font-family: ui-monospace, SFMono-Regular, 'SF Mono', Menlo, Consolas, 'Liberation Mono', monospace;
}

/* Scrollbar Utilities */
@layer utilities {
  .scrollbar-thin {
    scrollbar-width: thin;
    scrollbar-color: rgb(203 213 225) transparent;
  }

  .scrollbar-thin::-webkit-scrollbar {
    width: 6px;
    height: 6px;
  }

  .scrollbar-thin::-webkit-scrollbar-track {
    background: transparent;
  }

  .scrollbar-thin::-webkit-scrollbar-thumb {
    background-color: rgb(203 213 225);
    border-radius: 3px;
  }

  .scrollbar-thin::-webkit-scrollbar-thumb:hover {
    background-color: rgb(148 163 184);
  }
}

/* Typography Utilities */
.font-body {
  font-family: var(--font-body);
}

.font-heading {
  font-family: var(--font-heading);
}

.font-brand {
  font-family: var(--font-brand);
  letter-spacing: 0.5px;
}

/* Animation Utilities */
.animate-pulse-soft {
  animation: pulse-soft 8s cubic-bezier(0.4, 0, 0.6, 1) infinite;
}

.animate-fadeIn {
  animation: fadeIn 200ms ease-in;
}

.animate-slideUp {
  animation: slideUp var(--animation-normal) ease-out;
}

.animate-scaleIn {
  animation: scaleIn var(--animation-normal) ease-out;
}

@keyframes scaleIn {
  0% { transform: scale(0.95); opacity: 0; }
  100% { transform: scale(1); opacity: 1; }
}

/* Custom Border Color Utilities */
.border-primary-10 {
  border-color: var(--color-border-primary-10) !important;
}

.border-primary-15 {
  border-color: var(--color-border-primary-15) !important;
}

.border-primary-20 {
  border-color: var(--color-border-primary-20) !important;
}

/* Z-Index Utilities */
.z-modal {
  z-index: var(--z-modal) !important;
}

.z-tooltip {
  z-index: var(--z-tooltip) !important;
}

/* Modal Size Utilities */
.modal-sm {
  max-width: 28rem !important;
}

.modal-md {
  max-width: 42rem !important;
}

.modal-lg {
  max-width: 56rem !important;
}

.modal-xl {
  max-width: 90vw !important;
}

@media (min-width: 1920px) {
  .modal-xl {
    max-width: 1600px !important;
  }
}

/* Mobile Overrides */
@media (max-width: 767px) {
  @theme {
${generateMobileOverrides(theme)}
  }
}

/* Touch-Optimized Scrollbar */
@media (pointer: coarse) {
  @layer utilities {
    .scrollbar-thin::-webkit-scrollbar {
      width: 8px;
      height: 8px;
    }
    .scrollbar-thin::-webkit-scrollbar-thumb {
      background-color: rgb(148 163 184);
      border-radius: 4px;
    }
  }
}

/* Safe Area Support - Components handle their own safe areas */
@supports (padding: env(safe-area-inset-top)) {
  .mobile-nav {
    padding-bottom: calc(env(safe-area-inset-bottom) + 8px);
  }

  .mobile-header {
    padding-top: calc(env(safe-area-inset-top) + 8px);
  }
}

/* Horizontal Scroll Utilities */
@layer utilities {
  .overflow-x-auto-touch {
    overflow-x: auto;
    overflow-y: hidden;
    -webkit-overflow-scrolling: touch;
    scrollbar-width: thin;
  }

  .overflow-x-auto-touch::-webkit-scrollbar {
    height: 8px;
  }

  .overflow-x-auto-touch::-webkit-scrollbar-track {
    background: transparent;
  }

  .overflow-x-auto-touch::-webkit-scrollbar-thumb {
    background-color: rgb(203 213 225);
    border-radius: 4px;
  }

  .overflow-x-auto-touch::-webkit-scrollbar-thumb:hover {
    background-color: rgb(148 163 184);
  }

  .table-container {
    width: 100%;
    overflow-x: auto;
    -webkit-overflow-scrolling: touch;
  }

  .table-container table {
    min-width: 100%;
    width: max-content;
  }
}
`;
}

function generateTypeScript(theme, metadata) {
  return `/**
 * GENERATED FILE - DO NOT EDIT MANUALLY
 * Generated from: crates/services/web/config.yml and metadata.yml
 * Run 'npm run theme:generate' to regenerate
 */

export const theme = {
  branding: {
    name: '${theme.branding.name}',
    title: '${theme.branding.title}',
    description: '${theme.branding.description}',
    themeColor: '${theme.branding.themeColor}',
    logo: ${JSON.stringify(theme.branding.logo, null, 6)},
    favicon: '${theme.branding.favicon}',
  },
  metadata: ${JSON.stringify(metadata, null, 2)},
  colors: {
    light: ${JSON.stringify(theme.colors.light, null, 2)},
    dark: ${JSON.stringify(theme.colors.dark, null, 2)},
  },
  fonts: ${JSON.stringify(theme.fonts, null, 2)},
  typography: ${JSON.stringify(theme.typography, null, 2)},
  spacing: ${JSON.stringify(theme.spacing, null, 2)},
  radius: ${JSON.stringify(theme.radius, null, 2)},
  shadows: ${JSON.stringify(theme.shadows, null, 2)},
  animation: ${JSON.stringify(theme.animation, null, 2)},
  zIndex: ${JSON.stringify(theme.zIndex, null, 2)},
  layout: ${JSON.stringify(theme.layout, null, 2)},
  card: ${JSON.stringify(theme.card, null, 2)},
  mobile: ${JSON.stringify(theme.mobile, null, 2)},
  touchTargets: ${JSON.stringify(theme.touchTargets, null, 2)},
  navigation: ${JSON.stringify(theme.navigation, null, 2)},
} as const;

export type Theme = typeof theme;
`;
}

function main() {
  try {
    console.log('🎨 Generating theme from YAML...');

    const yamlContent = readFileSync(YAML_PATH, 'utf8');
    const theme = yaml.load(yamlContent);

    console.log('📖 Reading metadata...');
    const metadataContent = readFileSync(METADATA_PATH, 'utf8');
    const metadata = yaml.load(metadataContent);

    console.log('🔍 Validating theme schema...');
    const schemaContent = readFileSync(SCHEMA_PATH, 'utf8');
    const schema = JSON.parse(schemaContent);

    const ajv = new Ajv({ allErrors: true });
    const validate = ajv.compile(schema);
    const valid = validate(theme);

    if (!valid) {
      console.error('❌ Theme validation failed:');
      validate.errors.forEach(error => {
        console.error(`  - ${error.instancePath || '/'}: ${error.message}`);
        if (error.params) {
          console.error(`    Details: ${JSON.stringify(error.params)}`);
        }
      });
      process.exit(1);
    }
    console.log('✅ Theme validation passed');

    console.log('📝 Writing CSS...');
    const cssContent = generateCSS(theme);
    mkdirSync(dirname(CSS_OUTPUT), { recursive: true });
    writeFileSync(CSS_OUTPUT, cssContent);
    console.log(`✅ Generated: ${CSS_OUTPUT}`);

    console.log('📝 Writing TypeScript config...');
    const tsContent = generateTypeScript(theme, metadata);
    mkdirSync(dirname(TS_OUTPUT), { recursive: true });
    writeFileSync(TS_OUTPUT, tsContent);
    console.log(`✅ Generated: ${TS_OUTPUT}`);

    console.log('✨ Theme generation complete!');
  } catch (error) {
    console.error('❌ Error generating theme:', error.message);
    process.exit(1);
  }
}

main();
