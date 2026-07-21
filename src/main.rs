//! Entry point for the Enterprise Demo binary.
//!
//! Thin by design: every capability is registered at compile time by the
//! extension crates under `extensions/`, and this delegates to the published
//! `systemprompt` core runtime.

use systemprompt_template as _;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    Box::pin(systemprompt_template::cli::run()).await
}
