//! Opening pages in the user's default browser, for the sign-in flows of
//! `sp-accounts`.
//!
//! Copyright (c) systemprompt.io — Business Source License 1.1.
//! See <https://systemprompt.io> for licensing details.

use sp_accounts::{Browser, BrowserError};
use url::Url;

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct SystemBrowser;

impl Browser for SystemBrowser {
    fn open(&self, url: &Url) -> Result<(), BrowserError> {
        tracing::debug!(
            host = url.host_str().unwrap_or_default(),
            "opening the browser"
        );
        open::that_detached(url.as_str()).map_err(BrowserError)
    }
}
