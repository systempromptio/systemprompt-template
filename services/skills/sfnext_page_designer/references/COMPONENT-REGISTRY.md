# Component Registry Reference

## Overview

The component registry maps Page Designer `typeId` values to React components. The registry is **auto-generated** at build time by the staticRegistry Vite plugin.

## Static Registry

The file `src/lib/static-registry.ts` is generated automatically. **Do not edit it by hand.** The Vite plugin scans for components with `@Component` decorators and builds the registry.

After adding or modifying decorators, rebuild the app to regenerate the registry:

```bash
pnpm build
# or during development
pnpm dev   # Hot reload regenerates automatically
```

## Component Registration

Components are registered by their `@Component` decorator `typeId`:

```typescript
// The registry maps typeId -> component module
// Example auto-generated entry:
{
    'hero-banner': () => import('@/components/hero-banner'),
    'product-carousel': () => import('@/components/product-carousel'),
    'content-card': () => import('@/components/content-card'),
}
```

## Component Loaders

Components that need server data export a `loader` function. The registry calls these loaders during `collectComponentDataPromises()` in the route loader:

```typescript
// In the component file:
export function loader({ componentData, context }) {
    const clients = createApiClients(context);
    const productIds = componentData.productIds;

    return clients.shopperProducts.getProducts({
        params: { query: { ids: productIds.join(',') } }
    }).then(({ data }) => data);
}

export function fallback() {
    return <ProductCarouselSkeleton />;
}

export default function ProductCarousel({ data, ...props }) {
    // data = resolved result from loader
    return <div>{data.products.map(p => <ProductTile key={p.id} product={p} />)}</div>;
}
```

### Keep server-only loader code out of the client bundle (build gotcha)

In this repo a component's loader usually lives in a sibling `loaders.ts` that imports server-only modules (`@/lib/api/*.server`, `createApiClients`, or another lib that transitively does, e.g. `@/lib/api/content`). `index.tsx` re-exports it so the registry can find it:

```typescript
// loaders.ts — has server-only deps
import { fetchCategory } from '@/lib/api/categories.server';
export const loader = { server: async (args) => { /* ... */ } };

// index.tsx
export const loader = loaders.server;   // registry entry carries { loader: 'loader' }
```

The client build **strips `export const loader`** and dead-code-eliminates its now-unused import — **but only when the whole import declaration becomes unused**. If the `loader` value import shares one declaration with a still-used `type` (or another value) from the same loaders module, the declaration survives, the loaders module stays in the client graph, and its transitive `.server` import fails the build:

> [commonjs--resolver] Server-only module referenced by client — '…/categories.server' imported by 'src/components/…/loaders.ts'

```typescript
// ❌ co-located type keeps the declaration alive → .server leaks → build fails
import { loader as loaders, type FooLoaderData } from '@/components/foo/loaders';

// ✅ isolate the loader value on its own line so the stripper can drop it
import type { FooLoaderData } from '@/components/foo/loaders';
// eslint-disable-next-line no-duplicate-imports -- separate value import lets the client loader-stripper drop it, preventing a .server leak
import { loader as loaders } from '@/components/foo/loaders';
```

The two-imports split trips `no-duplicate-imports` (the repo has no `allowSeparateTypeImports`), hence the disable. A cleaner alternative, used by the einstein-recommender-* components, is to source the `LoaderData` type from a separate non-server module and import the loader value-only. Import-specifier form (relative `./loaders` vs absolute `@/components/foo/loaders`) is **irrelevant** — both resolve to the same module; only the split matters.

### Variant: runtime export from `loaders.ts` (cannot be fixed by import-splitting)

If `loaders.ts` exports a **runtime value** (not just a `type`) that the client component genuinely needs — e.g. `readMarkupField` in `pd/rich-text/loaders.ts` — no import-splitting trick saves you. The client needs that export at runtime, so the module stays in the client graph, drags `.server` imports with it, and the build fails with the same `Server-only module referenced by client` error. Symptom in dev: the Vite overlay points at `loaders.ts:<line>` on the very first `.server` import.

**Rule:** `loaders.ts` may export only `loader` + erased types (`interface`, `type`). Any runtime utility used by `index.tsx` (markup readers, formatters, helpers) belongs in a sibling client-safe module (e.g. `markup-utils.ts`). The loader file then imports the utility too if it needs it — that direction is fine.

```typescript
// ❌ runtime export keeps loaders.ts in the client bundle
// loaders.ts
export function readMarkupField(v: unknown): string { /* ... */ }   // runtime value
export const loader = { server: dataLoader };                       // server-only chain

// index.tsx
import { loader as loaders, readMarkupField } from './loaders';     // pulls .server into client
```

```typescript
// ✅ split runtime utility into a client-safe sibling
// markup-utils.ts (no .server imports anywhere in its graph)
export function readMarkupField(v: unknown): string { /* ... */ }
export interface FooLoaderData { /* ... */ }

// loaders.ts
import { fetchContentAssetById } from '@/lib/page-designer/fetch-content-asset.server';
export const loader = { server: dataLoader };
export type { FooLoaderData } from './markup-utils';   // type re-export is erased, fine

// index.tsx
import { loader as loaders } from './loaders';
import { readMarkupField, type FooLoaderData } from './markup-utils';
```

**Why a `loaders.ts` that exports ONLY `loader` + types works:** RR7's client loader-stripper drops `export const loader`, types are erased, and the whole module becomes dead-code-eliminated from the client bundle — its `.server` imports never need to resolve client-side.

### Even pure-type imports keep `loaders.ts` in the dev import graph

`verbatimModuleSyntax: true` is on, so TS/esbuild erase both `import type { X }` and `import { type X }` in emitted code. But Vite's dev **import-analyzer** runs on source before erasure, and still resolves the module path to walk its imports for the HMR graph. If `loaders.ts` has any `.server` import, the dev overlay fires `Server-only module referenced by client` pointing at `loaders.ts:<line>` even when the client file's only reference is a `type` import.

**Rule of thumb:** if `index.tsx` references `loaders.ts` for *anything* (a type, a value, anything), and `loaders.ts` imports `.server` anywhere in its graph, the dev build will fail. Always put `LoaderData` interfaces in a sibling `types.ts` (no `.server` imports anywhere downstream); have `loaders.ts` re-export the type from `types.ts`; have `index.tsx` import the type from `types.ts` directly.

```typescript
// types.ts (client-safe)
export interface FooLoaderData { /* ... */ }

// loaders.ts
import { something } from '@/lib/foo.server';
import type { FooLoaderData } from './types';
export type { FooLoaderData } from './types';   // re-export for callers that import from loaders
export const loader = { server: dataLoader };

// index.tsx — type imported from the client-safe module, not from loaders.ts
import type { FooLoaderData } from './types';
```

### Data Flow

```
Route loader
  |-- fetchPageFromLoader(args, { pageId })     -> page promise
  +-- collectComponentDataPromises(args, page)  -> componentData map
        |
  For each component with a loader:
    component.loader({ componentData, context }) -> data promise
        |
  <Region> renders components with resolved data
```

## Adding a New Page Designer Component

1. Create the React component with decorator metadata class
2. Implement the component's render function
3. (Optional) Export `loader` and `fallback` if server data is needed
4. Rebuild to regenerate static registry
5. Generate metadata JSON via MCP tool (`storefront_next_generate_page_designer_metadata`)
6. Deploy cartridge via MCP tool (`cartridge_deploy`)
