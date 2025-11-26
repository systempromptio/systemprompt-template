/**
 * Theme System Usage Examples
 *
 * This file demonstrates how to use the SystemPrompt theming system
 */

import React from 'react';
import { useTheme } from '@/theme';
import { BrandLogo, BrandText } from '@/components/brand';
import { Button } from '@/components/ui/Button';

export const ThemeExample: React.FC = () => {
  const { theme } = useTheme();

  return (
    <div className="min-h-screen bg-background text-text-primary p-xl">
      {/* Header with Brand Components */}
      <header className="mb-xl">
        <BrandLogo width={200} height={60} className="mb-md" />
        <BrandText size="large" className="block mb-sm" />
        <p className="text-text-secondary text-sm">
          A unified theming system for web and mobile
        </p>
      </header>

      {/* Color Palette */}
      <section className="mb-xl">
        <h2 className="font-heading text-xl mb-md">Color Palette</h2>
        <div className="grid grid-cols-2 md:grid-cols-4 gap-md">
          <ColorSwatch color="bg-primary" label="Primary" />
          <ColorSwatch color="bg-secondary" label="Secondary" />
          <ColorSwatch color="bg-success" label="Success" />
          <ColorSwatch color="bg-warning" label="Warning" />
          <ColorSwatch color="bg-error" label="Error" />
          <ColorSwatch color="bg-surface" label="Surface" dark />
          <ColorSwatch color="bg-surface-variant" label="Surface Variant" dark />
          <ColorSwatch color="bg-background" label="Background" dark />
        </div>
      </section>

      {/* Typography */}
      <section className="mb-xl">
        <h2 className="font-heading text-xl mb-md">Typography</h2>
        <div className="space-y-sm bg-surface dark:bg-surface-dark p-md rounded-lg">
          <p className="font-body text-xxl">Heading XXL (28px) - OpenSans</p>
          <p className="font-heading text-xl">Heading XL (20px) - Zepto</p>
          <p className="font-body text-lg">Large Text (18px) - OpenSans</p>
          <p className="font-body text-md">Body Text (15px) - OpenSans</p>
          <p className="font-body text-sm">Small Text (14px) - OpenSans</p>
          <p className="font-body text-xs">Extra Small (12px) - OpenSans</p>
          <p className="font-brand text-lg">Brand Text - ArchivoBlack</p>
        </div>
      </section>

      {/* Components */}
      <section className="mb-xl">
        <h2 className="font-heading text-xl mb-md">Components</h2>
        <div className="space-y-md">
          {/* Buttons */}
          <div className="flex gap-sm flex-wrap">
            <Button variant="primary" size="md">Primary Button</Button>
            <Button variant="secondary" size="md">Secondary Button</Button>
            <Button variant="success" size="md">Success Button</Button>
            <Button variant="destructive" size="md">Destructive Button</Button>
            <Button variant="ghost" size="md">Ghost Button</Button>
          </div>

          {/* Cards */}
          <div className="grid grid-cols-1 md:grid-cols-3 gap-md">
            <Card title="Default Card" shadow="md" />
            <Card title="Accent Shadow" shadow="accent" />
            <Card title="Large Shadow" shadow="lg" />
          </div>

          {/* Input */}
          <input
            type="text"
            placeholder="Themed input field"
            className="w-full px-md py-sm border border-border rounded-md
                       bg-surface text-text-primary
                       focus:outline-none focus:ring-2 focus:ring-primary
                       transition-fast"
          />
        </div>
      </section>

      {/* Spacing Scale */}
      <section className="mb-xl">
        <h2 className="font-heading text-xl mb-md">Spacing Scale</h2>
        <div className="space-y-xs bg-surface dark:bg-surface-dark p-md rounded-lg">
          <SpacingDemo size="xs" label="Extra Small (4px)" />
          <SpacingDemo size="sm" label="Small (8px)" />
          <SpacingDemo size="md" label="Medium (16px)" />
          <SpacingDemo size="lg" label="Large (24px)" />
          <SpacingDemo size="xl" label="Extra Large (32px)" />
          <SpacingDemo size="xxl" label="XXL (48px)" />
        </div>
      </section>

      {/* Using Theme Values Directly */}
      <section>
        <h2 className="font-heading text-xl mb-md">Using Theme Values</h2>
        <div
          className="p-md rounded-lg"
          style={{
            backgroundColor: theme.colors.surface,
            color: theme.colors.textPrimary,
            border: `1px solid ${theme.colors.border}`,
          }}
        >
          <p className="mb-sm">This component uses theme values directly:</p>
          <code className="block text-sm font-mono bg-background p-sm rounded">
            backgroundColor: theme.colors.surface
            <br />
            color: theme.colors.textPrimary
            <br />
            border: theme.colors.border
          </code>
        </div>
      </section>
    </div>
  );
};

const ColorSwatch: React.FC<{ color: string; label: string; dark?: boolean }> = ({
  color,
  label,
  dark,
}) => (
  <div className="text-center">
    <div className={`${color} h-20 rounded-md mb-xs shadow-sm`} />
    <p className={`text-xs ${dark ? 'text-text-secondary' : ''}`}>{label}</p>
  </div>
);

const Card: React.FC<{ title: string; shadow: 'sm' | 'md' | 'lg' | 'accent' }> = ({
  title,
  shadow,
}) => (
  <div className={`bg-surface dark:bg-surface-dark p-md rounded-lg shadow-${shadow}`}>
    <h3 className="font-medium mb-sm">{title}</h3>
    <p className="text-text-secondary text-sm">
      This card uses the {shadow} shadow variant.
    </p>
  </div>
);

const SpacingDemo: React.FC<{ size: string; label: string }> = ({ size, label }) => (
  <div className="flex items-center gap-md">
    <div className={`bg-primary h-md w-${size}`} />
    <span className="text-sm text-text-secondary">{label}</span>
  </div>
);

export default ThemeExample;
