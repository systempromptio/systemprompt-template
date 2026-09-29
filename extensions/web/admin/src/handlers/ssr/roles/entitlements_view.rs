//! The Entitlements tab: what each role opens, grouped by role and then by
//! entity kind, with a kind collapsed to one line when the role reaches
//! every entity of that kind the same way.
//!
//! The flat ledger writes `gateway_route/*` as one row per route, so a
//! rule that means "every model route" arrives here as a dozen rows; the
//! catalog says how many of each kind exist, and a role that names them all
//! with one access is shown as "All model routes (12)" with the rows
//! underneath for whoever wants them.

use serde::Serialize;

use super::view::RoleEntitlementView;
use crate::handlers::ssr::entity_kind::{entity_kind_plural, entity_kind_rank};
use crate::repositories::users::access_control::SectionInput;
use crate::types::Role;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct EntitlementKindView {
    pub kind: String,
    pub kind_label: &'static str,
    pub count: usize,
    pub catalog_size: usize,
    pub all_of_kind: bool,
    pub access: &'static str,
    pub access_tone: &'static str,
    pub summary: String,
    pub rows: Vec<RoleEntitlementView>,
    pub href: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct RoleEntitlementGroupView {
    pub role: String,
    pub role_label: String,
    pub tone: &'static str,
    pub total: usize,
    pub kinds: Vec<EntitlementKindView>,
}

type KindBuckets = Vec<(String, Vec<RoleEntitlementView>)>;

fn role_rank(role: &str) -> usize {
    Role::ALL
        .iter()
        .position(|r| r.as_str() == role)
        .unwrap_or(usize::MAX)
}

fn kind_view(
    kind: String,
    rows: Vec<RoleEntitlementView>,
    catalog_size: usize,
) -> EntitlementKindView {
    let allows = rows.iter().filter(|r| r.access == "allow").count();
    let denies = rows.len() - allows;
    let (access, access_tone) = match (allows, denies) {
        (_, 0) => ("allow", "ok"),
        (0, _) => ("deny", "err"),
        _ => ("mixed", "warn"),
    };
    let all_of_kind = catalog_size > 0 && rows.len() >= catalog_size && access != "mixed";
    let summary = if all_of_kind {
        format!(
            "All {} ({})",
            entity_kind_plural(&kind).to_lowercase(),
            rows.len()
        )
    } else if denies == 0 {
        format!("{allows} of {} allowed", catalog_size.max(rows.len()))
    } else if allows == 0 {
        format!("{denies} of {} refused", catalog_size.max(rows.len()))
    } else {
        format!("{allows} allowed, {denies} refused")
    };
    EntitlementKindView {
        kind_label: entity_kind_plural(&kind),
        href: format!("/admin/access-control?entity_kind={kind}&band=role"),
        kind,
        count: rows.len(),
        catalog_size,
        all_of_kind,
        access,
        access_tone,
        summary,
        rows,
    }
}

pub(super) fn groups(
    rows: Vec<RoleEntitlementView>,
    sections: &[SectionInput],
) -> Vec<RoleEntitlementGroupView> {
    let catalog_size = |kind: &str| {
        sections
            .iter()
            .find(|(k, _, _)| k == kind)
            .map_or(0, |(_, _, entities)| entities.len())
    };
    // Why: role → kind → rows, in first-seen order; sorted once below.
    let mut groups: Vec<(String, KindBuckets)> = Vec::new();
    for row in rows {
        let at = groups.iter().position(|(r, _)| *r == row.role);
        let at = at.unwrap_or_else(|| {
            groups.push((row.role.clone(), Vec::new()));
            groups.len() - 1
        });
        let kinds = &mut groups[at].1;
        match kinds.iter_mut().find(|(k, _)| *k == row.entity_type) {
            Some((_, list)) => list.push(row),
            None => kinds.push((row.entity_type.clone(), vec![row])),
        }
    }
    groups.sort_by_key(|(role, _)| role_rank(role));
    groups
        .into_iter()
        .map(|(role, mut kinds)| {
            kinds.sort_by_key(|(k, _)| entity_kind_rank(k));
            let kinds: Vec<EntitlementKindView> = kinds
                .into_iter()
                .map(|(kind, mut rows)| {
                    rows.sort_by(|a, b| {
                        a.entity_label
                            .to_lowercase()
                            .cmp(&b.entity_label.to_lowercase())
                    });
                    let size = catalog_size(&kind);
                    kind_view(kind, rows, size)
                })
                .collect();
            RoleEntitlementGroupView {
                role_label: role
                    .parse::<Role>()
                    .map_or_else(|()| role.clone(), |r| r.label().to_owned()),
                tone: match role.as_str() {
                    "platform_admin" => "err",
                    "admin" => "warn",
                    _ => "info",
                },
                total: kinds.iter().map(|k| k.count).sum(),
                role,
                kinds,
            }
        })
        .collect()
}
