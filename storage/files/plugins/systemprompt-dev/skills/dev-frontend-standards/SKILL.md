---
name: "Frontend Standards"
description: "Complete JavaScript, CSS, accessibility, event architecture, bundle pipeline, and admin UI coding standards"
---

# systemprompt.io Frontend Standards

**systemprompt.io is a world-class Rust programming brand.** Every frontend file must be instantly recognizable as idiomatic modern JavaScript as Addy Osmani would write it: performance-conscious, pattern-driven, zero-abstraction-debt.

Run `just publish` after changes. Verify zero console errors on every admin page.

---

## 1. Idiomatic JavaScript

Prefer functional composition, destructuring, and declarative patterns over imperative control flow.

```javascript
const names = items.filter(item => item.active).map(item => item.name);
const value = opt ?? computeDefault();
const { id, name, enabled = true } = config;
const endpoint = `/plugins/${encodeURIComponent(id)}`;
```

| Anti-Pattern | Idiomatic |
|--------------|-----------|
| `for (var i = 0; ...)` loop building array | `.filter().map()` chain |
| `if (x !== null && x !== undefined)` | `x ?? fallback` |
| `str + str + str` concatenation | Template literals `` `${a}/${b}` `` |
| `function(x) { return x.id; }` | Arrow `x => x.id` |
| Nested `if/else` chains | Early returns or ternary |
| Manual DOM string building | Template literal with `app.escapeHtml()` |
| `arr.indexOf(x) !== -1` | `arr.includes(x)` |
| Callback pyramids | `async`/`await` with `try`/`catch` |

---

## 2. Limits

| Metric | Limit |
|--------|-------|
| Source file length | 200 lines |
| Function length | 40 lines |
| Parameters | 4 |
| Nesting depth | 3 levels |
| Cyclomatic complexity | 10 per function |
| Document-level event listeners per module | 0 (use event hub) |
| DOM queries per function | 3 (cache references) |

---

## 3. Forbidden Constructs

| Construct | Resolution |
|-----------|------------|
| `document.addEventListener(...)` | Register via `app.events.on(type, selector, handler)` |
| `var` | Use `const` (preferred) or `let` |
| `==` / `!=` | Use `===` / `!==` |
| `innerHTML` with user content | Use `textContent` or `app.escapeHtml()` + `innerHTML` |
| Inline comments (`//`) | Delete -- code documents itself through naming |
| Block comments (`/* */`, `/** */`) | Delete -- no JSDoc, no section markers |
| TODO / FIXME / HACK comments | Fix immediately or don't write |
| `console.log` / `console.warn` / `console.error` | Use `app.Toast.show()` or remove |
| Global scope (no IIFE) | Wrap in `(function(app) { 'use strict'; ... })(window.AdminApp)` |
| Magic numbers / strings | Use named constants at module top |
| Commented-out code | Delete -- git has history |
| `setTimeout` for sequencing | Use events, promises, or `requestAnimationFrame` |
| `stopPropagation()` / `stopImmediatePropagation()` | Never needed with centralized event hub |
| Raw `fetch()` | Use `app.api()` for all API calls |
| `alert()` / `confirm()` / `prompt()` | Use `app.shared.showConfirmDialog()` or `app.Toast` |
| `eval()` / `new Function()` | Forbidden |

---

## 4. Mandatory Patterns

### Module Pattern

Every file uses the IIFE pattern with strict mode:

```javascript
(function(app) {
    'use strict';

    const API_PATH = '/plugins';

    function initPluginsPage() {
        app.events.on('click', '[data-action="delete"]', handleDelete);
        app.events.on('change', '[data-action="toggle"]', handleToggle);
    }

    async function handleDelete(e, el) {
        const id = el.dataset.entityId;
        app.shared.showConfirmDialog('Delete plugin?', 'This cannot be undone.', 'Delete', async () => {
            try {
                await app.api(`${API_PATH}/${encodeURIComponent(id)}`, { method: 'DELETE' });
                app.Toast.show('Plugin deleted', 'success');
                window.location.reload();
            } catch (err) {
                app.Toast.show('Failed to delete plugin', 'error');
            }
        });
    }

    app.initPluginsPage = initPluginsPage;
})(window.AdminApp);
```

### Event Hub

All interactivity via the centralized event hub. Zero direct `addEventListener` on `document`:

```javascript
app.events.on('click', '.action-trigger', handleActionClick, { exclusive: true });
app.events.on('click', '[data-delete]', handleDelete);
app.events.on('change', '.toggle-switch input', handleToggle);
```

### Error Handling

All async operations wrapped in `try`/`catch` with user feedback:

```javascript
async function deleteItem(id) {
    try {
        await app.api(`/items/${encodeURIComponent(id)}`, { method: 'DELETE' });
        app.Toast.show('Item deleted', 'success');
    } catch (err) {
        app.Toast.show('Failed to delete item', 'error');
    }
}
```

### HTML Security

All user-generated content through `app.escapeHtml()` before insertion via `innerHTML`:

```javascript
container.innerHTML = `<span class="user-name">${app.escapeHtml(user.name)}</span>`;
```

### API Calls

Always via `app.api()`. Never raw `fetch`:

```javascript
const data = await app.api('/plugins');
await app.api(`/plugins/${id}`, { method: 'PUT', body: JSON.stringify(payload) });
await app.api(`/plugins/${id}`, { method: 'DELETE' });
```

---

## 5. Naming

### Functions

| Prefix | Purpose | Returns |
|--------|---------|---------|
| `init` | Page initialization entry point | void |
| `render` | Creates/updates DOM | void |
| `handle` | Event handler callback | void |
| `fetch` | API data retrieval | Promise |
| `create` | Constructs new entity | Promise |
| `update` | Modifies existing entity | Promise |
| `delete` | Removes entity | Promise |
| `is` / `has` | Boolean check | boolean |
| `toggle` | Switches binary state | void |

### Variables

| Type | Convention | Example |
|------|-----------|---------|
| Functions, variables | `camelCase` | `handleClick`, `userName` |
| Constants | `SCREAMING_SNAKE_CASE` | `MAX_RETRIES`, `API_PATH` |
| DOM elements | Suffix with `El` or role | `containerEl`, `submitBtn` |
| Booleans | Prefix with `is`/`has`/`should` | `isOpen`, `hasPermission` |

### Abbreviations

Allowed: `id`, `url`, `api`, `btn`, `el`, `fn`, `ctx`, `req`, `res`, `msg`, `err`, `cfg`, `img`, `nav`, `col`, `idx`, `mcp`, `sse`

---

## 6. Accessibility

Every interactive element must be keyboard-accessible and screen-reader-friendly.

### Required ARIA Attributes

| Element | Required |
|---------|----------|
| Icon-only buttons | `aria-label="descriptive text"` |
| Toggle buttons (open/close) | `aria-expanded="true/false"` |
| Menu triggers | `aria-haspopup="true"` + `aria-expanded` |
| Loading states | `aria-busy="true"` on container |
| Live regions (toasts) | `role="alert"` or `aria-live="polite"` |
| Modals/dialogs | `role="dialog"` + `aria-modal="true"` + `aria-labelledby` |

### Rules

| Rule | Rationale |
|------|-----------|
| All `<button>` elements need accessible names | Screen readers announce button purpose |
| SVG icons need `aria-hidden="true"` | Prevents screen readers from reading SVG markup |
| Color is never the only indicator | Use icons, text, or patterns alongside color |
| Interactive elements are `<button>` or `<a>` | Never attach click handlers to `<div>` or `<span>` |
| Minimum touch target: 44x44px | Ensures usability on touch devices |

---

## 7. CSS Standards

### Layer System

All CSS files must declare their `@layer`. The cascade order is defined once in `tokens.css`:

```css
@layer tokens, reset, base, components, utilities, responsive;
```

### Forbidden CSS

| Construct | Resolution |
|-----------|------------|
| `!important` | Fix specificity with layers or more specific selector |
| Hardcoded colors | Use `var(--color-*)` from `tokens.css` |
| Hardcoded spacing | Use `var(--space-*)` from `tokens.css` |
| Hardcoded font sizes | Use `var(--text-*)` from `tokens.css` |
| Hardcoded border radius | Use `var(--radius-*)` from `tokens.css` |
| Hardcoded shadows | Use `var(--shadow-*)` from `tokens.css` |
| Vendor prefixes | Browser support is modern-only |
| Inline styles in templates | Use CSS classes |
| `float` for layout | Use flexbox or grid |
| `id` selectors for styling | Use class selectors |
| Commented-out CSS | Delete -- git has history |

### Naming Convention

BEM-lite: `component-element` with state modifiers via class composition.

| Pattern | Example |
|---------|---------|
| Component | `.install-widget` |
| Element | `.install-widget-header` |
| State | `.install-widget.open` |
| Global state | `.is-active`, `.is-loading` |
| JavaScript hook | `data-action="delete"` (never style `data-*`) |

---

## 8. Build Pipeline

### Publish Flow

```bash
just publish
```

Runs in strict order:

1. `bundle_admin_css` -- Concatenate CSS per `CSS_MODULE_ORDER` in Rust
2. `bundle_admin_js` -- Concatenate JS per `bundle-order.txt` + per-page bundles
3. `copy_extension_assets` -- Copy bundles to `web/dist/`
4. `content_prerender` -- Generate static content pages

Admin pages are SSR'd at runtime from `.hbs` templates in `storage/files/admin/templates/`; there is no precompile step.

### Adding New Files

| File Type | Steps |
|-----------|-------|
| New JS module | 1. Create file. 2. Add to `bundle-order.txt`. 3. Add to `bundles/*.txt`. 4. `just publish` |
| New CSS module | 1. Create file with `@layer`. 2. Add to `CSS_MODULE_ORDER` in Rust. 3. `just build && just publish` |

---

## 9. Pre-Publish Checklist

Before every `just publish`:

1. No `document.addEventListener` outside `events.js`, `sidebar-toggle.js`, `login.js`, `dashboard-sse.js`
2. No `var` declarations
3. No files exceed 200 lines
4. No comments of any kind
5. All CSS files use `@layer`
6. No hardcoded colors, spacing, font sizes in CSS
7. No `console.log`, `console.warn`, or `console.error`
8. All async operations have `try`/`catch` with user feedback
9. All user content escaped with `app.escapeHtml()`
10. All icon-only buttons have `aria-label`
11. All SVG icons have `aria-hidden="true"`

### Detection Commands

```bash
rg 'document\.addEventListener' storage/files/js/admin/ --glob '!events.js' --glob '!sidebar-toggle.js' --glob '!login.js' --glob '!dashboard-sse.js' --glob '!admin-*.js'
rg '\bvar\b ' storage/files/js/admin/ --glob '!admin-*.js'
rg 'console\.(log|warn|error)' storage/files/js/admin/ --glob '!admin-*.js'
rg '!important' storage/files/css/admin/
rg '^\s*//' storage/files/js/admin/ --glob '!admin-*.js'
```
