# State Management Skill

This skill covers client-side state management in Storefront Next with **server-first** data loading
and **React Router 7 primitives**. Business data lives on the server (`loader`/`action`); the browser
holds only ephemeral UI state, expressed with React context providers.

## Overview

Storefront Next keeps business data on the server and uses client state only for UX continuity.
App-wide state (auth/basket UI) and feature-local UI state (a modal's open/search state) are both
exposed through **React context providers** — the built-in mechanism, zero extra dependencies.

| State Type            | Mechanism                          | Example                                                |
| --------------------- | ---------------------------------- | ------------------------------------------------------ |
| Server data           | React Router `loader`              | Product details, category listings                     |
| App-wide client state | React context provider             | Session display state, basket snapshot/hydration state |
| Feature client state  | React context provider (`useReducer`/`useState`) | Store locator modal/search state          |
| Mutations             | React Router `action` (server)     | Add to cart, update profile                            |

## Feature-Local State — React context provider

For feature-scoped UI state (e.g. a store-locator modal), create a small context with a provider
backed by `useReducer` (or `useState` for trivial cases), and expose a typed hook:

```tsx
// src/extensions/store-locator/context/store-locator-context.tsx
import { createContext, useContext, useMemo, useReducer, type ReactNode } from 'react';

type StoreLocatorState = {
    isOpen: boolean;
    mode: 'input' | 'device';
    selectedStoreInfo: SelectedStoreInfo | null;
};

type StoreLocatorAction =
    | { type: 'open' }
    | { type: 'close' }
    | { type: 'setSelectedStoreInfo'; info: SelectedStoreInfo };

function reducer(state: StoreLocatorState, action: StoreLocatorAction): StoreLocatorState {
    switch (action.type) {
        case 'open':
            return { ...state, isOpen: true };
        case 'close':
            return { ...state, isOpen: false };
        case 'setSelectedStoreInfo':
            return { ...state, selectedStoreInfo: action.info };
        default:
            return state;
    }
}

type StoreLocatorContextValue = StoreLocatorState & {
    open: () => void;
    close: () => void;
    setSelectedStoreInfo: (info: SelectedStoreInfo) => void;
};

const StoreLocatorContext = createContext<StoreLocatorContextValue | null>(null);

export function StoreLocatorProvider({
    children,
    initial,
}: {
    children: ReactNode;
    initial?: Partial<StoreLocatorState>;
}) {
    const [state, dispatch] = useReducer(reducer, {
        isOpen: false,
        mode: 'input',
        selectedStoreInfo: initial?.selectedStoreInfo ?? null,
    });

    const value = useMemo<StoreLocatorContextValue>(
        () => ({
            ...state,
            open: () => dispatch({ type: 'open' }),
            close: () => dispatch({ type: 'close' }),
            setSelectedStoreInfo: (info) => dispatch({ type: 'setSelectedStoreInfo', info }),
        }),
        [state],
    );

    return <StoreLocatorContext.Provider value={value}>{children}</StoreLocatorContext.Provider>;
}

export function useStoreLocator() {
    const ctx = useContext(StoreLocatorContext);
    if (!ctx) throw new Error('useStoreLocator must be used within StoreLocatorProvider');
    return ctx;
}
```

Wrap only the subtree that needs it (e.g. the store-locator route or layout) with the provider, so
the state stays scoped and does not re-render the whole app.

## App-Level Context Integration

Expose app-wide state to components via providers/hooks (same pattern, mounted near the root):

```tsx
import { useBasket, useBasketSnapshot } from '@/providers/basket';

function CartIcon() {
    const basket = useBasket();
    const snapshot = useBasketSnapshot();

    const itemCount = basket?.productItems?.length ?? snapshot?.uniqueProductCount ?? 0;

    return <Badge count={itemCount} />;
}
```

## Post-Mutation Sync Pattern

Keep mutations on the server and update request-context resources there; the router revalidates
loaders so the UI reflects the new state without a manual client store:

```typescript
import { data } from 'react-router';
import { getBasket, updateBasketResource } from '@/middlewares/basket.server';

export async function action({ request, context }: ActionFunctionArgs) {
    const formData = await request.formData();
    const productId = formData.get('productId') as string;

    const basketResource = await getBasket(context);
    const clients = createApiClients(context);

    const { data: updatedBasket } = await clients.basket.addItemToBasket({
        params: { path: { basketId: basketResource.current?.basketId ?? '' } },
        body: { productId, quantity: 1 },
    });

    // Sync basket resource in request context for current response / revalidation flow
    updateBasketResource(context, updatedBasket);

    return data({ success: true, basket: updatedBasket });
}
```

## Best Practices

1. **Server-first data** — Load/mutate commerce data with `loader`/`action`; let revalidation refresh the UI.
2. **Provider-first state** — Use React context for both app-wide and feature-local UI state.
3. **Scope providers** — Mount a feature provider around only the subtree that needs it.
4. **`useReducer` for multi-field state** — Prefer a reducer over many `useState` calls when a feature has several related fields/transitions.
5. **Sync in server actions** — Update server basket/auth resources inside `action` handlers.
6. **Keep state minimal** — Store only what cannot be derived cheaply from loader data.

## When to Use Each Mechanism

| Scenario                      | Use                                                      |
| ----------------------------- | -------------------------------------------------------- |
| Product data on page load     | `loader`                                                 |
| Shopping cart badge count     | Basket provider hooks (`useBasket`, `useBasketSnapshot`) |
| Complex extension UI workflow | Feature context provider (`useReducer`)                  |
| Search results                | `loader`                                                 |
| Add to cart                   | Server `action` + resource update                        |

## Related Skills

- `sfnext_data_fetching` - Server-side data loading with loaders (NOT client state)
- `sfnext_authentication` - Auth state management
- `sfnext_components` - Using state in components
