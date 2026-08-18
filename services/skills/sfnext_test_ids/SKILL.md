# Add data-testid to Storefront Next elements

This skill defines the `data-testid` conventions for Storefront Next. Apply it whenever a new rendered element is introduced or an existing one needs to become test-addressable from unit tests or Playwright e2e. Adding stable selectors **while building the component** (shift-left) is cheaper than retrofitting them when tests are written.

> Paths and examples below are illustrative (`src/components/...`). Adapt them to the project's actual component tree. Confirm the real layout from a sibling file before editing.

## Decision: should this element get a `data-testid`?

Add a `data-testid` only when the element matches one of the categories below **and** a current or planned test (spec, unit test, or e2e step) will select or assert on it. Otherwise, prefer accessible queries (`getByRole` + accessible name, `getByLabelText`, `getByText`).

| Add testid | Skip testid |
| --- | --- |
| Outermost wrapper of a feature/component (root container) | Pure layout/spacing wrappers (`flex`, `grid`, `space-y-*` only) |
| Interactive controls — submit/CTA buttons, change/remove/add buttons | Decorative icons that are not asserted |
| `<form>` elements | Text spans inside an already-labelled container |
| Inputs without a visible label, with i18n-volatile labels, or with non-unique labels across the page | Inputs already uniquely queryable via `getByLabelText` / `getByRole('textbox', { name })` |
| Repeated items in a list (use dynamic suffix) | Snapshot-only divs / nested fragments |
| Status badges, indicators, counters | `<Link>` with unique text |
| Modal / dialog / sheet / popover roots | Tailwind utility-only divs that wrap a single child |
| Skeleton roots and major skeleton blocks | |
| Tabs, tab triggers, tab panels | |
| Empty-state and not-found containers | |
| Anything tests currently select by class name (replace it) | |

## Naming rules

**Format**: lowercase **kebab-case** only. No prefixes, no underscores, no camelCase, no spaces. The testid is the feature/element name in plain kebab-case; dynamic portions are appended via template literals.

**Picking the name**:

- **Root container** → `<feature>` or `<feature>-container` / `<feature>-card`. Example: `cart-pickup-card`, `profile-card`, `order-confirmation-container`.
- **Interactive control** → `<verb>-<noun>`. Example: `add-to-cart`, `change-store-button`, `remove-item-${itemId}`.
- **Form root** → `<entity>-form`. Example: `customer-profile-form`, `customer-address-form`.
- **Status / display value** → `<entity>-<field>` or `<entity>-status-badge`. Example: `order-number`, `total-orders-text`, `order-status-badge`.

```tsx
<h1 data-testid="product-title" className="text-2xl font-medium tracking-tight">
    {product.name}
</h1>

<Link to="/" aria-label={t('logoAriaLabel')} data-testid="header-logo">
    <Logo />
</Link>

<div className="pb-4" data-testid="cart-pickup-card">…</div>
```

## Icons and SVG elements

Icons (`lucide-react` exports, custom `*Icon` components, raw `<svg>` nodes) MAY carry a `data-testid` when a test legitimately needs to assert their presence — e.g. distinguishing a logged-in vs guest user icon, or asserting a cart icon renders inside a generic Button.

```tsx
export default function CartBadgeIcon({ numberOfItems }: { numberOfItems: number }) {
    return (
        <>
            <CartIcon className="size-5" data-testid="shopping-cart-icon" />
            {numberOfItems > 0 && (
                <Badge variant="default" data-testid="shopping-cart-badge">
                    {numberOfItems}
                </Badge>
            )}
        </>
    );
}
```

- Prefer asserting the **parent interactive control** (Button, Link, Badge) when role + accessible name uniquely identify it.
- Add a testid to the icon when the parent is generic (e.g. a `<Button variant="ghost">` whose only differentiator is the icon).
- Name icon testids `<feature>-icon` (e.g. `shopping-cart-icon`, `user-menu-icon`), kebab-case.
- Do NOT add a testid to purely decorative icons that are not referenced by any test — leave them `aria-hidden` and untagged.

## Inputs and form controls

The default is to query form controls by their **label** (`getByLabelText`) or **role + accessible name** (`getByRole('textbox', { name })`). Add a `data-testid` only when one of the conditions below applies. When you do, attach it on the **control itself** — the underlying `<input>`, `<textarea>`, `<select>`, checkbox/radio root — never on the surrounding `<FormItem>` or `<Label>` wrapper.

| Add testid | Skip testid |
| --- | --- |
| No visible label (icon-only search, password reveal, OTP) | Field rendered with `<Label htmlFor>` + visible label text |
| Label is i18n-volatile and tests must stay locale-stable | One field per page that the existing label already disambiguates |
| Multiple fields share the same label across the page (multi-ship address selects, per-row quantity inputs) | A single email/password field already addressable by `getByLabelText('Email')` |
| Inline / paragraph-embedded controls whose label is split across nodes (legal-text checkboxes) | Hidden inputs that no test references |
| Radio groups / native selects whose child items must be addressed by their parent | |
| Anything tests currently select by class name or `nth-child` (replace it) | |

### Naming pattern

`<feature>-<control-suffix>`. Pick the suffix from the control type:

| Control | Suffix | Examples |
| --- | --- | --- |
| Text / email / password / tel / generic input | `-input` (omit when feature name implies it) | `header-search`, `login-email-input`, `signup-password-input` |
| Textarea | `-textarea` | `contact-message-textarea`, `review-body-textarea` |
| Checkbox | `-checkbox` | `register-customer-checkbox`, `marketing-opt-in-checkbox` |
| Radio group root | `-radio` or `-select` (whichever reads naturally) | `delivery-option-select`, `shipping-speed-radio` |
| Native select | `-select` | `country-select`, `delivery-address-select` |
| OTP input | `-otp` | `passwordless-login-otp` |
| File input | `-file-input` | `avatar-file-input` |

For repeated controls, append a stable id template literal:

```tsx
<NativeSelect
    id={`delivery-address-select-${productItem?.itemId || index}`}
    data-testid={`delivery-address-select-${productItem?.itemId || index}`}
/>
```

### react-hook-form fields

When using the `<FormField>` / `<FormItem>` / `<FormControl>` / `<FormLabel>` family, the test id goes on the **rendered control** inside `<FormControl>`. `<FormItem>`, `<FormLabel>`, `<FormDescription>`, and `<FormMessage>` are layout/copy wrappers — they should NOT carry the field's testid.

```tsx
<FormField
    control={form.control}
    name="email"
    render={({ field }) => (
        <FormItem>
            <FormLabel>{t('emailLabel')}</FormLabel>
            <FormControl>
                <Input type="email" autoComplete="email" {...field} data-testid="signup-email-input" />
            </FormControl>
            <FormMessage />
        </FormItem>
    )}
/>
```

If the form-level error message is queried by tests, give the visible error its own testid: `data-testid="signup-email-error"`.

### Radio groups and selects

Place the testid on the **group/select root**, not on each option. Tests then use that root as a scope to query options by role + accessible name.

```tsx
<RadioGroup value={value} onValueChange={handleValueChange} data-testid="delivery-option-select">
    <RadioGroupItem value="ship" />
    <RadioGroupItem value="pickup" />
</RadioGroup>
```

```tsx
const radioGroup = screen.getByTestId('delivery-option-select');
const shipOption = within(radioGroup).getByRole('radio', { name: /ship/i });
```

If a particular option is itself dynamic (e.g. a per-store pickup row), add `data-testid={`delivery-option-${storeId}`}` on that `<RadioGroupItem>` only.

## Dynamic IDs (list items, repeated rows)

Use a template literal of the form `` `<feature>-${stableId}` ``. `stableId` MUST come from data: `productId`, `itemId`, `orderNo`, `id`, etc. Use array `index` only as a documented fallback when no stable id is available.

```tsx
<Card key={item.itemId ?? `item-${index}`} data-testid={`my-cart-item-${item.productId ?? index}`}>
    …
</Card>

<Button data-testid={`remove-item-${itemId}`} onClick={() => setShowConfirmation(true)}>
    {t('remove')}
</Button>
```

## Variant suffixes

- **Skeleton** — append `-skeleton` to the base name: `data-testid="product-skeleton"`.
- **Loading** (in-place loading variant of an existing testid) — append `-loading`: `data-testid="customer-address-form-loading"`.
- **Conditional / two-state** — pick a different name per state, do not toggle a boolean attribute:

```tsx
<button data-testid={expanded ? 'review-read-less' : 'review-read-more'}>
    {expanded ? t('readLess') : t('readMore')}
</button>
```

## Reusable components: accept the native `data-testid` attribute

If the component is reusable and callers need to disambiguate instances, declare the prop using the **native HTML attribute name** and destructure it with a local alias:

```tsx
type CardProps = {
    // …other props
    /** Test id for the root element */
    'data-testid'?: string;
};

function InfoCard({ title, 'data-testid': dataTestId = 'info-card' }: CardProps) {
    return <div data-testid={dataTestId}>{title}</div>;
}
```

Prop conventions:

- Prop name: **`'data-testid'?: string`** (the literal HTML attribute, quoted because of the dash). Do NOT introduce a camelCase `dataTestId` prop.
- Inside the component, destructure with a local alias and a sensible default so unit tests can query without the caller setting it.
- Apply on the **root** element only; do not double-apply.
- For compound components, derive child testids from the caller-provided base: `` data-testid={dataTestId ? `${dataTestId}-content` : undefined} ``.
- If the component already spreads a host element's props (`React.ComponentProps<'div'>` + `{...props}`), the `data-testid` already flows through — do not add a redundant prop declaration.

## shadcn `ui/*` primitives (if the project uses shadcn/ui)

Primitives that follow the shadcn convention spread `{...props}` and never declare a `dataTestId` prop.

**Consuming a `ui/*` primitive** — pass `data-testid` directly on the JSX element; the spread carries it to the host node:

```tsx
<Button onClick={handleSubmit} data-testid="submit-order">{t('submit')}</Button>
<Card data-testid="shipping-options-card">…</Card>
```

- Do **not** wrap the primitive in an extra `<div data-testid="…">` to attach the test id — apply it on the primitive.
- Do **not** add a `dataTestId` prop to a `ui/*` primitive.

**Authoring a new `ui/*` primitive** — declare it as `React.ComponentProps<'<tag>'>` (or the Radix equivalent) and spread `{...props}` so `data-testid` flows through. Add a `data-slot="<slot-name>"` structural marker (e.g. `data-slot="card"`, `data-slot="dialog-content"`).

### `data-slot` vs `data-testid`

`data-slot="…"` is **not** a substitute for `data-testid`. It is a stable structural marker baked into the primitive (every `<Card>` always renders `data-slot="card"`). Tests targeting a slot inside a feature should rely on the parent's `data-testid` plus the well-known `data-slot` child, e.g. `[data-testid="shipping-options-card"] [data-slot="card-content"]`. Do not add `data-testid` on every internal slot of a primitive.

## Placement inside JSX

Place `data-testid` **after** `className` on the same JSX element, on its own line if the line is long. Do not duplicate testids on a wrapper and its only child.

```tsx
<div className="store-inventory-filter …" data-testid="store-inventory-filter">
    …
</div>
```

## Workflow

Use this checklist for every component/route/extension target you touch:

```
- [ ] 1. Identify the root container — add a testid named after the feature (`<feature>-card`, `<feature>-container`).
- [ ] 2. Identify interactive controls (button, input, form) — add `<verb>-<noun>` or `<entity>-form` testids.
- [ ] 3. Identify repeated items — add `` `<feature>-${stableId}` `` testid on the row root.
- [ ] 4. Identify skeleton/loading variants — mirror the testid with `-skeleton` / `-loading` on the same base name.
- [ ] 5. If a shadcn `ui/*` primitive is the outermost element, attach `data-testid` directly on it — do not add an extra wrapper.
- [ ] 6. If the component is a custom reusable component, declare `'data-testid'?: string` (native attribute) and destructure with a local alias.
- [ ] 7. When authoring a new `ui/*` primitive, spread `{...props}` and add `data-slot="<slot-name>"`; do NOT introduce a `dataTestId` prop.
- [ ] 8. For icons asserted by tests, name the testid `<feature>-icon`; skip decorative icons no test references.
- [ ] 9. For form controls, add a testid only when the control lacks a stable visible label, has an i18n-volatile label, or repeats across the page. Apply it on the control itself — never on `<FormItem>` or `<Label>`. Name as `<feature>-<control-suffix>`.
- [ ] 10. Verify no duplicate testids in the same rendered tree (siblings + descendants).
- [ ] 11. Verify casing: kebab-case, no underscores, no spaces, no camelCase, no prefixes.
- [ ] 12. Verify dynamic ids use a stable data id (not array index alone).
- [ ] 13. Avoid testids that just duplicate role + accessible name; prefer that the test queries by role.
- [ ] 14. Run the project's typecheck and lint scripts after the edits.
```

## Anti-patterns to reject

- ❌ `data-testid="sf-cart-container"` — no namespace prefixes (`sf-`, `ui-`, `app-`); use plain kebab-case.
- ❌ `data-testid="ProductTile"` (PascalCase) / `data-testid="product_tile"` (snake_case) / `data-testid="add to cart"` (spaces).
- ❌ `` data-testid={`product-${index}`} `` when a `productId` is available — use the stable id.
- ❌ Adding a testid on a `<div>` that wraps a single `<button>` with an accessible name — query the button by role instead.
- ❌ Toggling testid with a boolean attribute (`data-testid={isOpen && 'modal'}`) — use a stable string or two distinct values.
- ❌ Reusing the same testid for two different things in the same render.
- ❌ Adding testids to elements only present as snapshot anchors.
- ❌ Declaring `dataTestId?: string` on a custom reusable component — use the native `'data-testid'?: string` destructured to a local alias.
- ❌ Adding a `dataTestId` (or `data-testid`) prop declaration to a shadcn `ui/*` primitive — they already forward `{...props}`.
- ❌ Wrapping a `ui/*` primitive in an extra `<div data-testid="…">` just to attach a testid.
- ❌ Treating `data-slot` as a `data-testid` (or replacing one with the other).
- ❌ Putting the form-field testid on `<FormItem>` / `<FormLabel>` / `<FormControl>` / `<Label>` instead of the underlying control.
- ❌ Adding a testid to a single labelled `<Input>` when `getByLabelText(t('label'))` already addresses it uniquely.
- ❌ Generic input testids like `email-input` reused across login, signup, and account forms — namespace with the feature (`login-email-input`, `signup-email-input`).
- ❌ Tagging every `<RadioGroupItem>` / `<option>` when the parent already has a testid — scope the query through the parent.

## Quick reference

| Element kind | Naming pattern | Example |
| --- | --- | --- |
| Feature root container | `<feature>` / `<feature>-card` / `<feature>-container` | `cart-pickup-card`, `profile-card` |
| Skeleton root | `<feature>-skeleton` | `product-skeleton`, `cart-skeleton` |
| List item root | `` `<feature>-${stableId}` `` | `` `my-cart-item-${productId}` `` |
| Loading variant | `<base>-loading` | `customer-address-form-loading` |
| Interactive button | `<verb>-<noun>` | `add-to-cart`, `remove-item-${itemId}` |
| Form root | `<entity>-form` | `customer-profile-form` |
| Text input (i18n-volatile / missing label) | `<feature>-input` (or `<feature>`) | `header-search`, `login-email-input` |
| Textarea | `<feature>-textarea` | `contact-message-textarea` |
| Checkbox | `<feature>-checkbox` | `marketing-opt-in-checkbox` |
| Radio group / native select root | `<feature>-select` / `<feature>-radio` | `delivery-option-select`, `country-select` |
| OTP input | `<feature>-otp` | `passwordless-login-otp` |
| Status badge | `<entity>-status-badge` | `order-status-badge` |
| Text / value display | `<entity>-<field>` | `order-number`, `total-orders-text` |
| Icon (when asserted) | `<feature>-icon` | `shopping-cart-icon` |
| Two-state toggle | descriptive per-state names | `review-read-more` / `review-read-less` |
| Custom reusable component prop | `'data-testid'?: string` → root only | destructure to `dataTestId` |
| shadcn `ui/*` (consumer) | pass `data-testid` natively on the JSX | `<Button data-testid="submit-order">` |
| shadcn `ui/*` (author) | spread `{...props}`, add `data-slot="<slot>"` | `card.tsx`, `dialog.tsx` |

## Related Skills

- `sfnext_components` — component authoring patterns these testids attach to.
- `playwright_cli` — driving the browser / generating e2e that consume these selectors (`getByTestId('…')`).
