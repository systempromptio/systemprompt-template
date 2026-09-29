//! The one place the Analysis section's URLs are spelled out.
//!
//! Platform pages answer "what is configured" at `/admin/<noun>`; Analysis
//! pages answer "how it performs" at `/admin/analysis/<noun>`, with the same
//! identity in the path: a conversation is its gateway context id, a skill
//! its `plugin:skill` key, a marketplace its id. Every link builder goes
//! through here so the two sections cannot drift apart again.

use systemprompt::identifiers::{ContextId, MarketplaceId};

pub(crate) const ANALYSIS_CONVERSATIONS_URL: &str = "/admin/analysis/conversations";
pub(crate) const ANALYSIS_SKILLS_URL: &str = "/admin/analysis/skills";
pub(crate) const ANALYSIS_VERSIONS_URL: &str = "/admin/analysis/versions";

pub(crate) fn analysis_conversation_url(context: &ContextId) -> String {
    format!(
        "{ANALYSIS_CONVERSATIONS_URL}/{}",
        urlencoding::encode(context.as_str())
    )
}

pub(crate) fn analysis_skill_url(skill_key: &str) -> String {
    format!("{ANALYSIS_SKILLS_URL}/{}", urlencoding::encode(skill_key))
}

pub(crate) fn analysis_version_url(marketplace: &MarketplaceId, hash: Option<&str>) -> String {
    let base = format!(
        "{ANALYSIS_VERSIONS_URL}/{}",
        urlencoding::encode(marketplace.as_str())
    );
    match hash {
        Some(h) => format!("{base}#v-{}", &h[..h.len().min(12)]),
        None => base,
    }
}
