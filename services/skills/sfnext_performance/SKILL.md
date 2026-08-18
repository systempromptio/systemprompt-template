# Performance Skill

This skill covers performance optimization techniques for Storefront Next storefronts.

## Bundle Size Limits

The application enforces strict bundle size limits in `package.json` under the `bundlesize` configuration.

```bash
pnpm bundlesize:test      # Verify bundle stays within limits
pnpm bundlesize:analyze   # Analyze bundle composition
```

## Built-in Metrics

Enable performance tracking in `config.server.ts`:

```typescript
{
    performance: {
        metrics: {
            serverPerformanceMetricsEnabled: true,
            clientPerformanceMetricsEnabled: true,
            serverTimingHeaderEnabled: false  // Enable for debugging only
        }
    }
}
```

Tracks:
- SSR operations and rendering time
- SCAPI API calls with parallelization visibility
- Authentication operations
- Client-side navigation timing

## Parallel Data Fetching

Return all promises simultaneously in loaders — avoid sequential `await`:

```typescript
// GOOD — Parallel (all requests start at once)
export function loader({ context }: LoaderFunctionArgs) {
    const clients = createApiClients(context);
    return {
        product: clients.shopperProducts.getProduct({...}),
        reviews: clients.shopperProducts.getReviews({...}),
        recommendations: clients.shopperProducts.getRecommendations({...}),
    };
}

// BAD — Sequential (each waits for previous)
export async function loader({ context }: LoaderFunctionArgs) {
    const product = await clients.shopperProducts.getProduct({...});
    const reviews = await clients.shopperProducts.getReviews({...});
    return { product, reviews };
}
```

## Image Optimization

Use the `DynamicImage` component with WebP format:

```typescript
import { DynamicImage } from '@/components/dynamic-image';

<DynamicImage
    src={product.image.link}
    alt={product.image.alt}
    width={400}
    height={400}
    format="webp"
/>
```

### Image Best Practices

- Use WebP format by default (smaller file sizes)
- Set explicit `width` and `height` to prevent layout shifts
- Lazy load below-the-fold images
- Use SCAPI image alt text as the primary alt source

## Progressive Streaming

Use synchronous loaders returning promises to stream data progressively:

```typescript
// Streams data as each promise resolves
export function loader({ context }: LoaderFunctionArgs) {
    const clients = createApiClients(context);
    return {
        product: clients.shopperProducts.getProduct({...}),    // Streams independently
        reviews: clients.shopperProducts.getReviews({...}),    // Streams independently
    };
}
```

Combine with granular Suspense boundaries for progressive page rendering.

## Lighthouse Optimization

Monitor and improve performance metrics:

```bash
pnpm lighthouse:ci   # Run Lighthouse CI
```

**Key areas:**
- Preload critical CSS
- Use WebP images by default
- Lazy load below-the-fold content
- Optimize font loading
- Minimize JavaScript bundle size

## Troubleshooting

| Issue | Cause | Solution |
|-------|-------|----------|
| Large bundle size | Unused imports or heavy dependencies | Run `bundlesize:analyze`; tree-shake or lazy load |
| Slow page transitions | Async loaders blocking | Use synchronous loaders returning promises |
| Layout shifts | Missing image dimensions | Set `width` and `height` on images |
| Slow SCAPI responses | Sequential API calls | Use parallel data fetching |

## Performance audit workflow

Measure first, change second, re-measure third — never optimize on intuition. Quote a real
metric (Lighthouse, WebPageTest, real-user data, bundle byte count) before proposing any code
change; "should be faster" is not a metric. Never invent web-perf APIs, Lighthouse fields,
React Router hooks, or Vite plugin names — verify against `package.json` versions and the
React Router docs, and mark anything unverifiable as such. For runtime questions ("does this
loader actually defer?", "what's in the entry chunk?") write a tiny probe (`node -e`, a
one-off Vitest, `pnpm vite build --debug`, `vite-bundle-visualizer` snapshot) under a scratch
directory and quote its output as evidence.

1. **Measure baseline.** Run Lighthouse against the affected route(s) on local dev AND the
   MRT preview (URL via `sfnext_deployment`). Capture LCP, CLS, INP, TBT, TTFB, total JS
   bytes, image bytes. Quote raw numbers — no rounding.
2. **Localize the cost.** Pick the most expensive culprit from the baseline:
   - LCP regression → above-the-fold loaders/components, image priority/sizes, font preload,
     critical CSS, SCAPI latency on the loader path.
   - CLS → images without `width`/`height`, `font-display: swap` without override
     descriptors, late-mounting components/skeletons with mismatched dimensions, embeds
     without reserved space, animations on `top`/`left`/`box-shadow`/`height`. A
     field-vs-lab CLS gap means post-load shifts; shifts within 500 ms of user input are
     excluded — don't chase them. Diagnose via DevTools shift bars, `web-vitals`
     attribution, or a PerformanceObserver probe; fix via reserved dimensions,
     hold-until-ready dimensional matching, and `@font-face` override descriptors.
   - INP/TBT → hydration cost (large client bundles), event handlers doing sync work,
     React 19 deferred-value misuse.
   - Bundle bloat → `vite-bundle-visualizer` or `pnpm vite build --mode=analyze`; look for
     duplicate deps, eagerly-imported heavy modules, missing `await import()` boundaries.
   - Slow streaming → a loader awaits something it shouldn't; check the streamed-promise
     pattern vs awaiting in the loader.
3. **Pick ONE change** — the smallest diff that targets the localized cost. Common moves
   (verify each against the current code first): move a data fetch from `await` to a
   streamed promise in the loader; `loading="eager" fetchpriority="high"` only on the LCP
   image and defer the rest; dynamic `import()` for below-the-fold routes/components;
   tighten image domain config so `$staticlink$` resolves without 404+retry on client
   navigation; replace a sync component with `Suspense` + `lazy` + skeleton.
4. **Re-measure.** Same Lighthouse config, same URL. Quote before/after with deltas. Reject
   the change if it doesn't move the targeted metric by a real amount (>5% or >100ms on the
   primary metric).
5. **Iterate, capped.** If still off-budget, return to step 2 — the top cost has likely
   shifted to a different culprit. Hard cap of 3 measure→change→re-measure cycles; if
   cycle 3 doesn't hit budget, stop and report the cumulative trace with options (accept
   partial gain, deeper architectural change, or out of budget).
6. **Verify no regressions.** Run the project's test / typecheck / lint scripts plus a
   Playwright smoke (`playwright_cli`). If you changed streaming or loader behavior, run the
   regression suite too.

**Anti-patterns to reject in review:** `React.memo` everywhere as a "perf" change (only
memoize when a profiler shows it matters); hand-rolled `useMemo`/`useCallback` on cheap
computations; eager-loading "to avoid jank" (usually trades INP for LCP); disabling
streaming to "fix" hydration warnings (fix the warning); adding caching layers without an
eviction story.

Done means: the primary metric improved by a quoted, repeatable amount on at least two
consecutive runs, no regression in other Core Web Vitals or test suites, and no new
`// TODO perf` debt.

## Related Skills

- `sfnext_data_fetching` - Parallel loader patterns for performance
- `sfnext_components` - Suspense boundaries for progressive rendering
- `sfnext_deployment` - Production build optimization
