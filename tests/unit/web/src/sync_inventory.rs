//! The configuration inventory names every kind the instance loads and
//! every path it names exists in this repository's `services/` tree; each
//! registered plane is a kind the inventory marks as projected.

use systemprompt_web_admin::repositories::sync::inventory::{
    CONFIG_KINDS, kind_by_id, kind_for_path,
};
use systemprompt_web_admin::repositories::sync::registry::planes;

use crate::support::repo_root;

#[test]
fn every_kind_exists_in_the_repository_tree() {
    let services = repo_root().join("services");
    for kind in CONFIG_KINDS {
        let path = services.join(kind.path);
        assert!(
            path.exists(),
            "inventory names services/{} but the tree has no such path",
            kind.path
        );
        assert_eq!(
            path.is_dir(),
            kind.is_dir,
            "services/{} kind mismatch",
            kind.path
        );
    }
}

#[test]
fn ids_are_unique_and_projected_kinds_come_first() {
    let mut seen = std::collections::HashSet::new();
    let mut saw_served = false;
    for kind in CONFIG_KINDS {
        assert!(seen.insert(kind.id), "duplicate kind id {}", kind.id);
        if kind.plane.is_some() {
            assert!(
                !saw_served,
                "projected kind {} listed after a served one",
                kind.id
            );
        } else {
            saw_served = true;
        }
    }
}

#[test]
fn every_registered_plane_is_a_projected_kind() {
    for plane in planes() {
        let kind = kind_for_path(plane.source_file())
            .unwrap_or_else(|| panic!("plane {} has no inventory kind", plane.id()));
        assert_eq!(
            kind.plane,
            Some(plane.id()),
            "kind {} is not marked projected",
            kind.id
        );
        assert_eq!(
            kind_by_id(plane.id()).map(|k| k.path),
            Some(plane.source_file())
        );
    }
}

#[test]
fn paths_resolve_to_the_exact_file_before_the_containing_directory() {
    assert_eq!(
        kind_for_path("web/config/groups.yaml").map(|k| k.id),
        Some("groups")
    );
    assert_eq!(
        kind_for_path("services/access-control/rules.yaml").map(|k| k.id),
        Some("access_control")
    );
    assert_eq!(
        kind_for_path("skills/who_am_i/SKILL.md").map(|k| k.id),
        Some("skills")
    );
    assert!(kind_for_path("evaluation/criteria/cost.md").is_none());
    assert!(kind_for_path("skillsx/config.yaml").is_none());
    assert!(kind_for_path("nowhere.yaml").is_none());
}
