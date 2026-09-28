//! The one form a skill has once it leaves `services/`.
//!
//! A skill is declared as `services/skills/<snake_id>/`, and every host
//! presents it dashed: Claude Code lists `/<plugin>:<dashed-id>`, the bridge
//! writes `skills/<dashed-id>/SKILL.md`, and the hooks report the dashed
//! name back. Analytics therefore key on the dashed `plugin:skill` — the
//! `analysis_skill_events` view, the fact pipeline and the marketplace hash
//! all agree on it — and anything that starts from a services id must go
//! through here rather than compare the two spellings.

// Why: `plugin:skill` as the hosts and the analytics tables spell it.
#[must_use]
pub fn skill_ref(plugin: &str, skill: &str) -> String {
    format!("{plugin}:{}", host_skill_name(skill))
}

// Why: the skill's name on a host — the services id with `_` as `-`.
#[must_use]
pub fn host_skill_name(skill: &str) -> String {
    skill.replace('_', "-")
}
