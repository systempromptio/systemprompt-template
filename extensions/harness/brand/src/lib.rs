//! Compile-time product identity per brand and channel.
//!
//! A build is one brand and one channel, both chosen by cargo features: the
//! brand by `brand-astound` (default `systemprompt`), the channel by
//! `channel-canary` or `channel-dev` (default `internal`). Every channel of
//! every brand has its own app id, URL scheme, data root and display name, so
//! any number of them install side by side and never read each other's
//! accounts, sessions or settings.
//!
//! [`identity()`] is the identity this binary was built as; [`Identity::of`]
//! and [`Identity::ALL`] expose the whole table so tooling can check that no
//! two builds collide.
//!
//! Copyright (c) systemprompt.io — Business Source License 1.1.
//! See <https://systemprompt.io> for licensing details.

use std::path::PathBuf;

mod brand;
mod channel;
mod identity;

pub use brand::Brand;
pub use channel::Channel;
pub use identity::{Identity, identity};

const VERSION: &str = "0.1.0";
const CONFIG_DIR_NAME: &str = ".harness";

// Why: Windows always has %LOCALAPPDATA%, but a stripped service account or a
// CI sandbox can lack a known folder; the profile's conventional location keeps
// the data root absolute when a home directory is known.
fn local_data_base() -> PathBuf {
    dirs::data_local_dir()
        .or_else(|| dirs::home_dir().map(|home| home.join("AppData").join("Local")))
        .unwrap_or_else(|| PathBuf::from("."))
}
