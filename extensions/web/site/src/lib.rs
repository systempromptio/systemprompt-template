//! Public-site page data providers for the web extension.
//!
//! Each module owns the data model for a section of the marketing/docs site
//! and exposes a `*PageDataProvider` that the core SSR runtime calls when
//! rendering. Queries are confined to `repositories`, which reads the
//! `markdown_content` tables populated by the content ingestion job.
//!
//! - [`homepage`], [`docs`] — section providers.
//! - [`navigation`] — header / footer nav config consumed by every page.
//! - [`partials`] / `partials_animations` — shared template fragments.
//! - [`extenders`] — URL extenders that splice org-specific routes onto the
//!   public surface.
//! - [`assets`] — `web_assets()` enumerates the static asset manifest for the
//!   extension trait.

pub mod assets;
pub mod config_loader;
pub mod docs;
pub mod extenders;
#[doc(hidden)]
pub mod format;
pub mod homepage;
pub mod navigation;
pub mod partials;
mod partials_animations;
mod repositories;
pub mod skills_page;

pub use assets::web_assets;
