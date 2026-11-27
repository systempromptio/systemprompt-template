//! Opening pages in the user's default browser, for the sign-in flows of
//! `sp-accounts`.

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
