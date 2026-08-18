# MRT Data Store — Site/Global Preferences Skill

How to correctly read MRT Data Store custom preferences (BM site/global prefs synced to MRT)
in a Storefront Next app — and avoid the two silent traps that make a preference read
`undefined`/`{}` only after deploy or after an SDK upgrade.

There are **two** gotchas, and they compound. Get both right.

## ⚠️ Gotcha 1 — the value is NOT a flat map (shape mismatch)

The official "MRT Data Store Usage Guide" shows preferences as a **flat map**:

```ts
const sitePrefs = getSitePreferences(context);
return { showBanner: Boolean(sitePrefs.showBanner) }; // ← top-level key access
```

This is **only true for the local dev seed**. The **real MRT Data Store** returns the entry
`value` as an **array of preference descriptors** under `data`:

```jsonc
// real MRT store value for "{siteId}-custom-site-preferences"
{ "data": [
  { "id": "gReCaptchaSiteKey", "value": "6LeDD_0s…", "groupId": "GoogleRecaptcha" },
  { "id": "showBanner",        "value": true,        "groupId": "Homepage" },
  // … many more descriptors for {siteId}
] }
```

The SDK does **not** flatten it. So `sitePrefs.gReCaptchaSiteKey` is `undefined` (the only
top-level key is `data`). Not an SDK bug, not a "custom key" problem — just the shape your
BM→MRT sync produces.

**Why it bites only in prod:** locally you seed with `MRT_DATA_STORE_DEFAULTS`, which is
**already flat**, so top-level access works and gives false confidence. The array shape
appears only against the real store on a deployed environment.

✅ **Fix:** flatten the entry once, in the middleware `transform` (`flattenSitePreferences`).

## ⚠️ Gotcha 2 — own the context AND the reader (SDK ≥ 1.0.0)

> Regression history: the Storefront Next **SDK 1.0.0** migration is known to break prefs
> site-wide. Pre-1.0.0 the SDK **exported** `sitePreferencesContext`, so a custom
> flatten-middleware could write into the same context the SDK's `getSitePreferences` reads.
> 1.0.0 made that context **private** (no longer exported). The migration replaced the
> import with a locally-created `createDataStoreContext()` — which compiles, typechecks and
> passes tests, but silently points the middleware at a context **nothing reads**.

`createDataStoreContext()` returns an **identity-keyed** React Router context (a fresh object
per call — `createContext(null)`). The SDK's own `getSitePreferences(context)` reads the SDK's
**private** context, which is populated **only** by the SDK's built-in
`customSitePreferencesMiddleware` (which has **no `transform`**). The moment you need a
`transform` (Gotcha 1), you run your **own** middleware writing into your **own** context —
and the SDK's `getSitePreferences` then reads a context you never populated → returns `{}`.

✅ **Fix:** the app **owns the whole trio** — context, middleware, and `getSitePreferences` —
in one module, and **every consumer imports the app's `getSitePreferences`, never the SDK's.**

```text
flatten middleware ──writes──▶ sitePreferencesContext ◀──reads── getSitePreferences
        (root.tsx)              (app-owned, one object)          (app module)
                          consumers import the APP getSitePreferences
```

## Correct setup — app owns context + reader + transform

```ts
// src/lib/data-store/site-preferences.ts — the single source of truth
import type { RouterContextProvider } from 'react-router';
import { createDataStoreContext, type SitePreferences } from '@salesforce/storefront-next-runtime/data-store';

// App-owned context. Consumers MUST read through getSitePreferences below — NOT the SDK's,
// which reads a different (private, never-populated-here) context and returns {}.
export const sitePreferencesContext = createDataStoreContext<SitePreferences>();

/** Reads the flattened site prefs; `{}` (fail-open) when the middleware hasn't run. */
export function getSitePreferences(context: Readonly<RouterContextProvider>): SitePreferences {
    return context.get(sitePreferencesContext) ?? ({} as SitePreferences);
}

/** Normalises the MRT entry value (array / envelope / flat) → flat `{ prefId: value }`. */
export function flattenSitePreferences(value: Record<string, unknown>): Record<string, unknown> {
    const data = value?.data;
    // 1) Real MRT store: array of { id, value, groupId } descriptors → { id: value }.
    if (Array.isArray(data)) {
        const flat: Record<string, unknown> = {};
        for (const entry of data) {
            if (entry && typeof entry === 'object') {
                const id = (entry as { id?: unknown }).id;
                if (typeof id === 'string') flat[id] = (entry as { value?: unknown }).value;
            }
        }
        return flat;
    }
    // 2) Object envelope: { data: { … } }.
    if (data && typeof data === 'object') return data as Record<string, unknown>;
    // 3) Already flat (dev pseudo-store seed).
    return value ?? {};
}
```

Register the middleware in `src/root.tsx`, importing the context **from the app module**
(not the SDK) and running it **after** `siteContextMiddleware` (the entry key needs the site id):

```ts
// src/root.tsx
import { createDataStoreMiddleware, type SitePreferences } from '@salesforce/storefront-next-runtime/data-store';
import { flattenSitePreferences, sitePreferencesContext } from '@/lib/data-store/site-preferences';

const sitePreferencesMiddleware = createDataStoreMiddleware<SitePreferences>({
    entryKey: (context) => `${context.get(siteContext)?.site?.id ?? ''}-custom-site-preferences`,
    context: sitePreferencesContext, // ← app-owned, the SAME object getSitePreferences reads
    transform: flattenSitePreferences, // ← normalises array/envelope/flat → flat map
    onUnavailable: 'fallback', // never throw at request time
    fallbackValue: {} as SitePreferences,
});

export const middleware: MiddlewareFunction[] = [
    // …
    siteContextMiddleware, // must run before site-prefs (resolves site id)
    sitePreferencesMiddleware,
    // …
];
```

## Reading a single preference (consumer pattern)

Consumers wrap each pref in a typed, fail-safe helper — and import `getSitePreferences`
from the **app module**, never from `@salesforce/storefront-next-runtime/data-store`:

```ts
// src/lib/turnstile/recaptcha/site-key.server.ts
import type { RouterContextProvider } from 'react-router';
import { getSitePreferences } from '@/lib/data-store/site-preferences'; // ← app module, NOT the SDK

/** MRT Data Store id mirroring the BM site pref. */
export const RECAPTCHA_SITE_KEY_PREF = 'gReCaptchaSiteKey';

export function getRecaptchaSiteKey(context: Readonly<RouterContextProvider>): string {
    const value = getSitePreferences(context)[RECAPTCHA_SITE_KEY_PREF];
    return typeof value === 'string' ? value : ''; // '' = "not configured / off"
}
```

Then surface it to the client via the loader (there is **no client-side prefs hook**):

```ts
export function loader({ context }: LoaderFunctionArgs) {
    return { recaptchaSiteKey: getRecaptchaSiteKey(context) };
}
// component receives it as loaderData and passes it down as a prop
```

> Lint guard idea: forbid importing `getSitePreferences` from the SDK in app code
> (`no-restricted-imports`), so every consumer is forced through the app module.

## What the official guide gets right (and the gaps)

| Topic | Official guide | Reality / what to add |
| --- | --- | --- |
| Where to read | Server middleware, loaders, actions | ✅ correct |
| Client access | No client hook — read in loader, pass via `useLoaderData`/props | ✅ correct |
| Return type | `Record<string, unknown>` (cast/validate at call site) | ✅ correct |
| **Value shape** | Implies flat map (`sitePrefs.showBanner`) | ❌ real store is `{ data: [{id,value,groupId}] }`; **add a `transform`** (Gotcha 1) |
| **Which `getSitePreferences`** | The SDK's | ❌ with a custom transform you must use the **app's** reader on the **app's** context (Gotcha 2) |
| Entry keys | Site: `{siteId}-custom-site-preferences`; Global: `custom-global-preferences` | ✅ correct |
| Unavailable mode | `onUnavailable: 'fallback'` | ✅ use it — fail open |

## Local development

Seed without a real store via `MRT_DATA_STORE_DEFAULTS`. Site prefs **must** be keyed with the
`{siteId}-` prefix; the seed is **flat**, which is why the array shape never appears locally:

```bash
MRT_DATA_STORE_DEFAULTS='{
  "custom-global-preferences": { "enableChat": true },
  "{siteId}-custom-site-preferences": { "gReCaptchaSiteKey": "test-key", "productsPerPage": 24 }
}' pnpm dev
```

Required env in prod (set by MRT): `AWS_REGION`, `MOBIFY_PROPERTY_ID`, `DEPLOY_TARGET`.

## Debugging "set in BM but undefined/empty after deploy (or after an SDK upgrade)"

Two failure signatures — tell them apart by logging in **two** places:

1. **In the middleware `transform`** (raw shape from MRT) — temporarily:
   `console.warn('[prefs] flatten', { topLevelKeys: Object.keys(value ?? {}), flatKeys: Object.keys(flat) })`.
   - Only top-level key is `data` (array) and `flat` has real ids → **Gotcha 1** (shape). The
     transform is doing its job; make sure it's wired.
2. **In the consumer** (`getSitePreferences` at read time):
   `logger.warn('[prefs] read', { keys: Object.keys(getSitePreferences(context)) })`.
   - Transform log shows the ids, but the consumer log shows `[]`/`{}` → **Gotcha 2**
     (context mismatch). The consumer is reading a different context than the middleware
     wrote — check it imports the **app** `getSitePreferences` and that `root.tsx` passes the
     **app** `sitePreferencesContext` into the middleware.
   - SDK also emits a `debug` "Data store context not found" on a miss (suppressed in prod) —
     a strong tell for Gotcha 2.
3. Fix, re-deploy, confirm the consumer log shows the resolved value (e.g. `6LeDD_0s…`).
4. **Remove the diagnostic logs** before shipping — pref helpers stay pure/side-effect-free.

## Do / Don't

- ✅ Own the trio in one module: context + `getSitePreferences` + `flattenSitePreferences`.
- ✅ Import `getSitePreferences` from the **app module** in every consumer.
- ✅ Pass the **app's** `sitePreferencesContext` into `createDataStoreMiddleware` in `root.tsx`.
- ✅ Flatten once in the `transform`; keep consumer helpers trivial.
- ✅ Treat a missing pref as "feature off" (`''` / `false`), never throw.
- ✅ Mirror the exact BM pref id in a single exported constant (`RECAPTCHA_SITE_KEY_PREF`).
- ✅ Verify against a **deployed** environment, not just local — the shapes differ.
- ❌ Don't import `getSitePreferences`/`sitePreferencesContext` from the SDK in app code —
  the SDK context is private (≥1.0.0) and never populated by your custom middleware.
- ❌ Don't assume `getSitePreferences(context)[prefId]` works just because it works locally.
- ❌ Don't read prefs from a client component — thread via loader/props.
- ❌ Don't leave diagnostic logging or `getLogger` in pref helpers.

## Common Pitfalls

| Pitfall | Problem | Solution |
| --- | --- | --- |
| Flat-map assumption | `sitePrefs[id]` is `undefined` on prod (real store is `{ data: [ … ] }`) | Add `transform: flattenSitePreferences` (Gotcha 1) |
| SDK reader + custom middleware | Consumer reads `{}` though the middleware ran (different contexts) | Own + import the app `getSitePreferences`; pass app context to the middleware (Gotcha 2) |
| Imported SDK context (pre-1.0.0 habit) | Breaks/compiles-to-wrong-context after SDK upgrade — export removed in 1.0.0 | Create and own the context in the app module |
| Local-only verification | Dev seed is flat → false confidence | Verify on a deployed env |
| Middleware order | Site-prefs entry key is empty | Register after `siteContextMiddleware` |
| Hard failure on outage | Page 500s when store is unreachable | `onUnavailable: 'fallback'`, `fallbackValue: {}` |
| Diagnostic log left in | Noisy/impure helper shipped | Remove temp logging + `getLogger` before commit |

## Reference implementation (sfnext, SDK 1.0.0)

- `sfnext/src/lib/data-store/site-preferences.ts` — app-owned `sitePreferencesContext`,
  `getSitePreferences`, and `flattenSitePreferences` (the owned trio)
- `sfnext/src/root.tsx` — `createDataStoreMiddleware` with `transform`, fed the app context
- `sfnext/src/lib/turnstile/recaptcha/site-key.server.ts` — typed single-pref consumer
  importing the **app** `getSitePreferences` (`getRecaptchaSiteKey`)
- Other consumers all import the app reader: `zendesk/snippet.server.ts`,
  `noibu-config.server.ts`, `bazaarvoice-config.server.ts`, `unbxd-search/.../suggest-prefs.server.ts`,
  `f5-shape/lib/site-prefs.ts`, `auth/passwordless-gating.ts`, `content-folder-navigation.server.ts`

## Related Skills

- `sfnext_configuration` — `config.server.ts` / `PUBLIC__` app config (different system)
- `sfnext_data_fetching` — using context in loaders/actions
- `generate-site-preferences-impex` — defining the BM site preferences that sync to the store
- `b2c_mrt` — Managed Runtime operations
