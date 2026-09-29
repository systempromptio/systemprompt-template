//! What changed between two marketplace versions, read from their stored
//! manifests alone: plugins and skills added, removed or changed by digest.
//! Nothing here touches a tree or the database, so a diff between any two
//! recorded versions is always available and always the same.

use std::collections::BTreeMap;

use serde::Serialize;
use systemprompt::identifiers::{PluginId, SkillId};

use crate::repositories::sync::marketplace_hash::MarketplaceManifest;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Change {
    Added,
    Removed,
    Changed,
    Unchanged,
}

impl Change {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Added => "added",
            Self::Removed => "removed",
            Self::Changed => "changed",
            Self::Unchanged => "unchanged",
        }
    }

    #[must_use]
    pub const fn tone(self) -> &'static str {
        match self {
            Self::Added => "ok",
            Self::Removed => "err",
            Self::Changed => "warn",
            Self::Unchanged => "muted",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SkillChange {
    pub plugin_id: PluginId,
    pub skill_id: SkillId,
    pub skill_key: String,
    pub change: Change,
    pub change_label: &'static str,
    pub change_tone: &'static str,
    pub before: Option<String>,
    pub after: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PluginChange {
    pub plugin_id: PluginId,
    pub change: Change,
    pub change_label: &'static str,
    pub change_tone: &'static str,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct ManifestDiff {
    pub plugins: Vec<PluginChange>,
    pub skills: Vec<SkillChange>,
    pub added: usize,
    pub removed: usize,
    pub changed: usize,
    pub unchanged: usize,
}

impl ManifestDiff {
    #[must_use]
    pub const fn has_changes(&self) -> bool {
        self.added + self.removed + self.changed > 0
    }

    #[must_use]
    pub fn moved_skills(&self) -> Vec<&SkillChange> {
        self.skills
            .iter()
            .filter(|s| s.change != Change::Unchanged)
            .collect()
    }
}

type SkillDigests<'a> = BTreeMap<(&'a str, &'a str), (&'a str, &'a str)>;

fn skill_digests(manifest: &MarketplaceManifest) -> SkillDigests<'_> {
    manifest
        .plugins
        .iter()
        .flat_map(|p| {
            p.skills.iter().map(move |s| {
                (
                    (p.plugin_id.as_str(), s.skill_id.as_str()),
                    (s.skill_key.as_str(), s.digest.as_str()),
                )
            })
        })
        .collect()
}

fn plugin_digests(manifest: &MarketplaceManifest) -> BTreeMap<&str, &str> {
    manifest
        .plugins
        .iter()
        .map(|p| (p.plugin_id.as_str(), p.digest.as_str()))
        .collect()
}

fn classify(before: Option<&str>, after: Option<&str>) -> Change {
    match (before, after) {
        (None, Some(_)) => Change::Added,
        (Some(_), None) => Change::Removed,
        (Some(a), Some(b)) if a == b => Change::Unchanged,
        (Some(_), Some(_)) => Change::Changed,
        (None, None) => Change::Unchanged,
    }
}

// Why: `before` is the older version. A skill present in both with a
// different digest is *changed*; a plugin whose own files are unchanged but
// whose skills moved is reported through those skills, not as a changed
// plugin, so the counts name what actually moved.
#[must_use]
pub fn diff(before: Option<&MarketplaceManifest>, after: &MarketplaceManifest) -> ManifestDiff {
    let empty = MarketplaceManifest {
        marketplace_id: after.marketplace_id.clone(),
        name: String::new(),
        version: String::new(),
        plugins: Vec::new(),
        files: 0,
    };
    let before = before.unwrap_or(&empty);
    let (old_p, new_p) = (plugin_digests(before), plugin_digests(after));
    let (old_s, new_s) = (skill_digests(before), skill_digests(after));

    let mut plugins = Vec::new();
    let mut plugin_ids: Vec<_> = old_p.keys().chain(new_p.keys()).collect();
    plugin_ids.sort_unstable();
    plugin_ids.dedup();
    for id in plugin_ids {
        let change = classify(old_p.get(id).copied(), new_p.get(id).copied());
        plugins.push(PluginChange {
            plugin_id: PluginId::new(*id),
            change,
            change_label: change.label(),
            change_tone: change.tone(),
        });
    }

    let mut out = ManifestDiff {
        plugins,
        ..ManifestDiff::default()
    };
    let mut skill_keys: Vec<_> = old_s.keys().chain(new_s.keys()).collect();
    skill_keys.sort_unstable();
    skill_keys.dedup();
    for key in skill_keys {
        let (old, new) = (old_s.get(key), new_s.get(key));
        let change = classify(old.map(|v| v.1), new.map(|v| v.1));
        match change {
            Change::Added => out.added += 1,
            Change::Removed => out.removed += 1,
            Change::Changed => out.changed += 1,
            Change::Unchanged => out.unchanged += 1,
        }
        out.skills.push(SkillChange {
            plugin_id: PluginId::new(key.0),
            skill_id: SkillId::new(key.1),
            skill_key: new.or(old).map_or_else(String::new, |v| v.0.to_owned()),
            change,
            change_label: change.label(),
            change_tone: change.tone(),
            before: old.map(|v| v.1.to_owned()),
            after: new.map(|v| v.1.to_owned()),
        });
    }
    out
}
