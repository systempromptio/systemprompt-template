# Tailwind v4 in Storefront Next

Storefront Next uses **Tailwind CSS v4** — CSS-first config (no `tailwind.config.js`), registered
as the `@tailwindcss/vite` plugin in `vite.config.ts`. This skill is a generic v4 + shadcn/ui
reference. Concrete design-system values (spacing scale, fonts, shadow values, brand colors) are
**per-project** and live in that project's `src/styles/` — always grep the project's tokens before
picking a value.

## Core discipline

- **Merge classes via `cn()`** — never concatenate `className` with `+` or template strings.

```tsx
import { cn } from '@/lib/utils'; // cn = twMerge(clsx(...))

<div className={cn('rounded-md p-4', isActive && 'ring-2 ring-ring', className)} />
```

`cn` resolves Tailwind conflicts (`p-2 p-4` → `p-4`) and skips falsy values. Pass `className`
through last so callers can override.

- **Tokens only, no raw colors.** Use semantic tokens (`bg-background`, `text-foreground`,
  `bg-primary`, `border-border`, `ring-ring`, …) not hardcoded palette utilities (`bg-red-500`,
  `text-gray-700`, `bg-white`). For one-off brand tokens use `bg-[var(--brand-...)]`. Many projects
  enforce this with a custom ESLint rule (e.g. `no-restricted-classnames` / `pnpm lint:colors`).

- **Dynamic class names must be complete strings at build time.** `bg-${color}-500` will NOT be
  generated — map prop → static class via a lookup table.

## Recommended CSS structure

Tailwind v4 reads everything from CSS. A clean split keeps the entry point tiny:

```css
/* src/app.css — entry point only */
@import 'tailwindcss';
@import 'tw-animate-css';                 /* optional animation utilities */

@custom-variant dark (&:is(.dark *));     /* class-based dark mode */

@import './styles/theme.css';     /* @theme inline { ... } — utility-generating tokens */
@import './styles/utilities.css'; /* @utility blocks, @keyframes */
@import './styles/tokens.css';    /* :root and .dark CSS-variable values */
@import './styles/base.css';      /* @layer base — resets, body */
@import './styles/components.css'; /* global selectors not expressible as utilities */
```

Order matters: `theme.css` must come **before** any `@utility` that references theme keys.
`tokens.css` (raw `:root` / `.dark` variable values) is independent.

Adding a new **color** token is 3 steps: (1) declare the raw value in `tokens.css`
(`:root { --foo: #...; }`, plus `.dark` if it differs), (2) expose it in `theme.css`
(`--color-foo: var(--foo);`) so `bg-foo`/`text-foo` become valid, (3) if the project lints colors,
add the token to the allowlist.

## Dark mode

- Apply `.dark` on a parent (usually `<html>`); token values flip in `.dark { ... }` in `tokens.css`.
- With tokens you don't repeat colors for dark mode — `bg-card text-card-foreground` adapts.
- Use `dark:` only when you need a different shape/opacity, not a different palette (`dark:bg-input/30`).
- Tokens identical in light and dark live **only in `:root`** and inherit via cascade.

## Responsive

Mobile-first, stock breakpoints (`sm/md/lg/xl/2xl` = 40/48/64/80/96rem):

```tsx
<div className="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-3" />
```

Use `max-*` to invert (`max-md:flex-col`) and one-off ranges (`min-[600px]:`, `max-[479px]:`).

## Variants with CVA (shadcn pattern)

```ts
const buttonVariants = cva('base classes', {
    variants: {
        variant: { default: '...', destructive: '...', outline: '...' },
        size: { default: '...', sm: '...', icon: 'size-9' },
    },
    defaultVariants: { variant: 'default', size: 'default' },
});

<Comp className={cn(buttonVariants({ variant, size, className }))} />
```

## Adding shadcn components

```bash
cd sfnext
npx shadcn@latest add <component-name>
```

- `src/components/ui/` holds **only** ejected shadcn components — never hand-copy shadcn source
  (use the CLI so `components.json` + deps stay consistent). Custom UI lives in `src/components/<feature>/`.

---

## Tailwind v4 reference

### CSS entry shape

Single `@import 'tailwindcss'` — no `@tailwind base / components / utilities` (removed in v4).

### Directives (CSS-only config)

| Directive | What it does |
|-----------|--------------|
| `@import "tailwindcss"` | Loads theme defaults, preflight, utilities in layers `theme, base, components, utilities` |
| `@theme { … }` | Registers design tokens **and** generates utilities; values reach utilities through CSS variables — overridable via cascade (`:root` / `.dark`) |
| `@theme inline { … }` | Same registration, but values are **inlined** into utilities at build time — cascade overrides DO NOT change the utility. Use when a token aliases another CSS variable in `:root` / `.dark` |
| `@theme static { … }` | Always emits all CSS variables (even unused) |
| `@custom-variant <name> (<selector>)` | Adds a variant (e.g. `dark (&:is(.dark *))`) |
| `@variant <name> { … }` | Apply a Tailwind variant inside custom CSS |
| `@utility <name>` (and `<name>-*`) | Register a custom utility; functional form supports `--value()` / `--modifier()` / `--default()` |
| `@apply <utility>…` | Inline a utility into custom CSS; OK in the entry/base, avoid in components |
| `@source "<path>"` / `@source inline("<class>")` | Add / safelist source paths and classes |
| `@reference "<path>"` | Import tokens for `<style>` blocks / CSS modules **without** emitting CSS |
| `@config "<file.js>"` / `@plugin "<pkg>"` | v3 compat (legacy JS config / plugin) |

### Theme namespaces → utilities (cheat sheet)

| Namespace | Generates utilities for |
|-----------|--------------------------|
| `--color-*` | `bg-*`, `text-*`, `border-*`, `ring-*`, `fill-*`, `stroke-*`, `decoration-*` |
| `--font-*` | `font-sans`, custom families |
| `--text-*` (+ optional `--text-*--line-height`) | `text-xs … text-9xl` |
| `--font-weight-*` | `font-medium`, `font-bold` |
| `--tracking-*` / `--leading-*` | `tracking-*` / `leading-*` |
| `--breakpoint-*` | `sm: md: lg: xl: 2xl:` |
| `--container-*` | container query variants `@sm: @md:` + `max-w-*` |
| `--spacing-*` (and base `--spacing`) | `p-*`, `m-*`, `w-*`, `h-*`, `gap-*` |
| `--radius-*` | `rounded-sm`, `rounded-lg`, custom names |
| `--shadow-*`, `--inset-shadow-*`, `--drop-shadow-*`, `--text-shadow-*` | `shadow-*`, `inset-shadow-*`, etc. |
| `--blur-*`, `--perspective-*`, `--aspect-*`, `--ease-*`, `--animate-*` | corresponding utilities |

To **disable a namespace**, set `--color-*: initial` (or `--*: initial` to clear all defaults) inside `@theme`.

### `@theme` vs `@theme inline`

- `@theme { --color-foo: <value>; }` — utility compiles to `background-color: var(--color-foo)`
  ⇒ **cascade override in `:root` / `.dark` works** (this is how you swap themable tokens between themes).
- `@theme inline { --color-foo: var(--foo); }` — Tailwind inlines `var(--foo)` into the utility
  ⇒ the reference is to `--foo`. Use when a theme token aliases another CSS variable in `:root` / `.dark`.

**Shadow exception** (`--shadow-*`, `--inset-shadow-*`, `--drop-shadow-*`, `--text-shadow-*`):
shadow utilities are inlined into `--tw-shadow` with `var(--tw-shadow-color, …)` spliced per layer,
so **redeclaring `--shadow-xs` in `:root` / `.dark` does NOT change the `.shadow-xs` utility**. To make
a shadow themable, override the utility itself while keeping the composition chain:

```css
/* tokens.css */
:root { --shadow-xs: 0 1px 2px 0 rgb(0 0 0 / 0.1); }
.dark { --shadow-xs: 0 1px 2px 0 rgb(255 255 255 / 0.06); }

/* utilities.css */
@utility shadow-xs { --tw-shadow: var(--shadow-xs); }
```

The custom `@utility` lands after the defaults and overwrites only `--tw-shadow`, so `:root`/`.dark`
cascade now switches the shadow at use time while `ring-*` / `inset-shadow-*` keep working.
Trade-off: multi-layer values without `var(--tw-shadow-color, …)` slots won't recolor via
`shadow-<color>/N`. Note `shadow-lg` etc. are also stock utilities — a custom `@utility shadow-lg`
overrides Tailwind's default everywhere the class is used (usually intentional, to match design).

### Build-time CSS functions (v4)

| Function | Purpose | Example |
|----------|---------|---------|
| `--alpha(color / N%)` | Adjust opacity via `color-mix(in oklab, …)` | `color: --alpha(var(--color-primary) / 50%);` |
| `--spacing(N)` | Generate spacing from `--spacing` base | `padding: --spacing(4);` |
| `--value(<theme-key>-*, <type>, [<type>])` | Inside `@utility name-*`, resolve arg against theme keys / bare / arbitrary types | `tab-size: --value(--tab-size-*, integer, [integer]);` |
| `--modifier(…)` | Same as `--value()` for the `/modifier` portion | `line-height: --modifier(--leading-*, [length]);` |
| `--default(<value>)` | Default when none given | `tab-size: --value(integer, --default(4));` |

### Arbitrary values & properties

- Arbitrary value: `top-[117px]`, `bg-[#bada55]`, `text-[22px]`
- CSS variable shorthand: `fill-(--my-color)` ⇔ `fill-[var(--my-color)]`
- Hint type when ambiguous: `text-(length:--my-var)`, `text-(color:--my-var)`
- Arbitrary property: `[mask-type:luminance]`; combinable with variants: `hover:[mask-type:alpha]`
- Whitespace inside arbitrary value: use `_` (`grid-cols-[1fr_500px_2fr]`); in JSX use `String.raw`

### Variants & dark mode

- Default `dark:` uses `prefers-color-scheme`. Override with `@custom-variant dark (&:is(.dark *))`
  to follow a `.dark` class on an ancestor (class-based dark mode).
- Compose in selectors: `[&[data-state=open]]:rotate-180`, `has-[input:checked]:bg-primary`,
  `aria-invalid:ring-destructive/20`.
- Breakpoint variants stock `sm/md/lg/xl/2xl`; `max-*` to invert.

### Container queries

- Mark parent with `@container` (`@container/main` to name it).
- Variants `@sm: @md:` (mirror breakpoint scale) and `@max-md:`; combine `@sm:@max-md:flex-col`.
- Container length units (`cqw`, `cqi`, `cqb`, `cqh`) usable as arbitrary values.

### Colors (Tailwind v4)

- Default palette is **OKLCH**, wider gamut on capable displays. Reference via `var(--color-name)` or tokens.
- Opacity modifier `/N` works on any color utility: `bg-primary/30`, `text-foreground/60` (compiled to `color-mix`).
- Define brand colors in `@theme` (or in `:root` and expose via `@theme inline`).

### v3 → v4 renamings (don't copy v3 docs blindly)

| v3 class | v4 class |
|----------|----------|
| `shadow` | `shadow-sm` |
| `shadow-sm` | `shadow-xs` |
| `rounded` | `rounded-sm` |
| `rounded-sm` | `rounded-xs` |
| `blur` / `blur-sm` | `blur-sm` / `blur-xs` |
| `outline-none` | `outline-hidden` (new `outline-none` sets `outline-style: none`) |
| `ring` | `ring-3` (default ring width became `1px`, color `currentColor`) |
| `flex-shrink-*` / `flex-grow-*` | `shrink-*` / `grow-*` |
| `bg-opacity-*`, `text-opacity-*`, … | `/opacity` modifiers (e.g. `bg-black/50`) |
| `bg-gradient-to-r` | `bg-linear-to-r` |

Other behaviour changes: `space-x/y-*` and `divide-x/y-*` now select `:not(:last-child)`; default
`border-*` color is `currentColor` (no implicit `gray-200` — always pair `border` with a color token);
container queries are first-class (no plugin).

### Detecting classes & sourcing

- `node_modules`, `.gitignore`d files, CSS, lockfiles, binaries are **skipped** by default.
- Include a 3rd-party lib: `@source "../node_modules/@org/lib";`
- Safelist: `@source inline("{hover:,focus:,}underline");` (brace expansion supported).
- Set scan root: `@import "tailwindcss" source("../src");` · disable auto-detect: `source(none)`.

### Functional utilities (template for design-system additions)

```css
@utility tab-* {
    tab-size: --value(--tab-size-*, integer, [integer]);
}
@utility opacity-* {
    opacity: calc(--value(integer) * 1%);
    opacity: --value(--opacity-*, [percentage]);
}
```

### Browser & runtime notes

- Targets **Safari 16.4+**, **Chrome 111+**, **Firefox 128+** (uses `@property`, `color-mix()`, container queries).
- The `@tailwindcss/vite` plugin handles HMR and CSS extraction — no `postcss.config.js` needed.
- Class **detection is plain-text** (no AST) — reinforces the `cn()` + static-class rule.
- Original CSS variables stay on `:root`, so JS can read them via
  `getComputedStyle(document.documentElement).getPropertyValue('--color-primary')`.

### Docs index

- Theme: https://tailwindcss.com/docs/theme
- Directives & functions: https://tailwindcss.com/docs/functions-and-directives
- Detecting classes / safelisting: https://tailwindcss.com/docs/detecting-classes-in-source-files
- Colors / OKLCH: https://tailwindcss.com/docs/colors
- Dark mode: https://tailwindcss.com/docs/dark-mode
- Upgrade guide (v3 → v4): https://tailwindcss.com/docs/upgrade-guide
- Vite install: https://tailwindcss.com/docs/installation/using-vite

## Related Skills

- `sfnext_components` - Building components that consume these tokens (shadcn, CVA)
- `sfnext_project_setup` - Where `src/styles/` and `vite.config.ts` are set up
