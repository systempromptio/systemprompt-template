#[must_use]
pub fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

pub mod playbook_categories {
    pub const GUIDE_PREFIX: &str = "guide-";
    pub const CLI_PREFIX: &str = "cli-";
    pub const BUILD_PREFIX: &str = "build-";
    pub const CONTENT_PREFIX: &str = "content-";
    pub const VALIDATION_PREFIX: &str = "validation-";

    pub const GUIDE: &str = "guide";
    pub const CLI: &str = "cli";
    pub const BUILD: &str = "build";
    pub const CONTENT: &str = "content";
    pub const VALIDATION: &str = "validation";
}
