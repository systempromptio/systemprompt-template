//! Public-site page data providers for the web extension.
//!
//! Each module owns the data model for a section of the marketing/docs site
//! and exposes a `*PageDataProvider` that the core SSR runtime calls when
//! rendering. No `sqlx` or DB access here — the site reads from prerendered
//! content artifacts produced by `systemprompt_web_jobs::ContentPrerenderJob`.
//!
//! - [`homepage`], [`blog`], [`docs`], [`features`] — section providers.
//! - [`navigation`] — header / footer nav config consumed by every page.
//! - [`partials`] / [`partials_animations`] — shared template fragments.
//! - [`extenders`] — URL extenders that splice org-specific routes onto
//!   the public surface.
//! - [`assets`] — `web_assets()` enumerates the static asset manifest for
//!   the extension trait.

pub mod assets;
pub mod blog;
pub mod docs;
pub mod extenders;
pub mod features;
pub mod homepage;
pub mod navigation;
pub mod partials;
mod partials_animations;
mod repositories;

pub use assets::web_assets;
