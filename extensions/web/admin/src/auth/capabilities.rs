use crate::types::UserContext;

/// Whether the caller is allowed to view raw, un-redacted transcript content.
///
/// This is the gate that protects PII (prompts, tool args, model outputs) from
/// being serialized into the DOM for users without a clearance to see it.
///
/// Until a richer capability layer exists, the rule is:
///   - admins (`is_admin == true`) always pass;
///   - any user holding the `auditor` role passes.
pub fn can_view_raw_transcript(user: &UserContext) -> bool {
    user.is_admin || user.roles.iter().any(|r| r.eq_ignore_ascii_case("auditor"))
}
