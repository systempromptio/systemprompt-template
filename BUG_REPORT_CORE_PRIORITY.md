# Bug Report: Core Default Prerenderer Priority Should Be Lower

## Summary

The core library's default homepage prerenderer has priority 100, which is higher than custom extension prerenderers. This causes a **priority inversion bug** where extensions cannot override the default without setting artificially high priorities.

## Current Behavior

```
Core default homepage prerenderer: priority 100
Custom HomepagePrerenderer:        priority 10
```

Result: Core's prerenderer wins (higher priority = wins), blocking custom prerenderers from executing. The homepage renders with empty template variables because the core's default provides no data.

## Expected Behavior

Extensions should be able to override core defaults with reasonable priority values. The core's defaults should act as fallbacks, not as overrides.

```
Core default homepage prerenderer: priority 10 (or lower)
Custom HomepagePrerenderer:        priority 50-100
```

## Workaround

Currently, extensions must set priority > 100 to override the core:

```rust
// extensions/web/src/homepage/prerenderer.rs
fn priority(&self) -> u32 {
    150 // Must be higher than core's default (100)
}
```

This is a hack that will break if core increases its priority.

## Recommended Fix

Lower the core's default prerenderer priority to 10 or below:

```rust
// In systemprompt core library
impl PagePrerenderer for DefaultHomepagePrerenderer {
    fn priority(&self) -> u32 {
        10 // Low priority - extensions should override
    }
}
```

## Evidence

Debug logs showing two prerenderers registered:
```
Registering page prerenderer page_type=homepage priority=100  <- core default (wins)
Registering page prerenderer page_type=homepage priority=10   <- custom (blocked)
```

## Impact

- Extensions cannot provide custom homepage data without knowing core's internal priority
- Default "fallback" behavior is inverted - core acts as override, not fallback
- Developers must use arbitrarily high priorities to work around the issue

## Related

This may affect other default prerenderers/providers in the core library. Consider reviewing all default priorities to ensure extensions can override them naturally.
