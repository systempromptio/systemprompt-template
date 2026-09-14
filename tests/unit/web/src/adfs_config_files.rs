//! The committed ADFS SSO configs are load-bearing and unvalidated until boot:
//! (`entity_id`/`acs_url` are normally absent and derived from the profile's
//! external URL; these checks cover an authored override.)
//! nothing else in the suite reads `services/web/config/adfs*.yaml`. These
//! checks parse every one of them the way the loader does, so a typo'd key, a
//! metadata file that is not there, or an ACS URL that disagrees with the
//! relying-party trust fails here rather than at a user's sign-in.

use std::collections::BTreeSet;
use std::path::PathBuf;

use systemprompt_web_admin::AdfsConfig;

use crate::support::repo_root;

const ACS_PATH: &str = "/admin/auth/adfs/acs";

fn config_dir() -> PathBuf {
    repo_root().join("services/web/config")
}

// Why: `adfs.yaml` plus any per-profile override beside it. The loader picks
// `adfs.<profile>.yaml` when present and falls back to `adfs.yaml`, so every
// file matching that shape is one some deployment boots from.
fn adfs_config_files() -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = std::fs::read_dir(config_dir())
        .expect("services/web/config is readable")
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|name| name.starts_with("adfs.") && name.ends_with(".yaml"))
        })
        .collect();
    files.sort();
    // Why: every gate below asserts a property of each file it finds, so an
    // empty list satisfies all of them. Renaming the configs would otherwise
    // turn this whole module green.
    assert!(
        !files.is_empty(),
        "no adfs.*.yaml under {}",
        config_dir().display()
    );
    files
}

fn parse(path: &PathBuf) -> AdfsConfig {
    let yaml = std::fs::read_to_string(path).expect("config file is readable");
    serde_yaml::from_str(&yaml).unwrap_or_else(|e| {
        panic!(
            "{} does not deserialise into AdfsConfig: {e}",
            path.display()
        )
    })
}

// Why: The scheme-and-host prefix, which is all the two URLs have to agree on.
fn origin(url: &str) -> Option<&str> {
    let (scheme, rest) = url.split_once("://")?;
    let host = rest.split('/').next()?;
    Some(&url[..scheme.len() + 3 + host.len()])
}

#[test]
fn every_committed_adfs_config_parses() {
    let files = adfs_config_files();
    assert!(
        !files.is_empty(),
        "no services/web/config/adfs*.yaml found; the default config is not optional"
    );
    for path in &files {
        let config = parse(path);
        assert_eq!(
            config.entity_id.is_empty(),
            config.acs_url.is_empty(),
            "{} sets one SAML URL and derives the other; they are derived together or authored together",
            path.display()
        );
    }
}

#[test]
fn every_enabled_config_pins_a_metadata_file_that_exists() {
    for path in adfs_config_files() {
        let config = parse(&path);
        if !config.enabled {
            continue;
        }
        let metadata = config_dir().join(&config.idp_metadata_path);
        assert!(
            metadata.is_file(),
            "{} names idp_metadata_path {} which does not exist",
            path.display(),
            metadata.display()
        );
    }
}

// Why: AD FS writes the registered ACS as both `Destination` and `Recipient`,
// and the SP rejects an assertion whose Destination disagrees. Shipping
// `/callback` against a trust registered for `/acs` cost us a 307 into a 405.
#[test]
fn the_acs_url_shares_the_entity_host_and_is_the_route_we_serve() {
    for path in adfs_config_files() {
        let config = parse(&path);
        // Why: both empty is the normal case — the loader derives them from
        // the profile's external URL, which is what keeps one config correct
        // on every deployment host. Only an authored override is checked here.
        if config.entity_id.is_empty() && config.acs_url.is_empty() {
            continue;
        }
        assert!(
            config.acs_url.ends_with(ACS_PATH),
            "{} sets acs_url {} but the route we serve is {ACS_PATH}",
            path.display(),
            config.acs_url
        );
        assert_eq!(
            origin(&config.entity_id),
            origin(&config.acs_url),
            "{} points entity_id and acs_url at different hosts",
            path.display()
        );
    }
}

// Why: `services/web/config/groups.yaml` is the only place an AD group is
// mapped onto a DB group or project, and the login gate records every group
// the assertion carried. A mapping naming a group no adfs config knows is
// therefore a mapping that can never fire — usually a typo in a
// `Systemprompt-*` name, which is invisible at runtime because an unmapped
// group is a legitimate state.
#[test]
fn every_mapped_ad_group_is_one_a_config_names() {
    let path = repo_root().join("services/web/config/groups.yaml");
    let doc: serde_yaml::Value = serde_yaml::from_str(
        &std::fs::read_to_string(&path).expect("web/config/groups.yaml is readable"),
    )
    .expect("web/config/groups.yaml parses");

    let mapped: BTreeSet<String> = adfs_config_files()
        .iter()
        .flat_map(|path| {
            let cfg = parse(path);
            cfg.group_roles
                .keys()
                .cloned()
                .chain(cfg.group_role_patterns.keys().cloned())
                .collect::<Vec<_>>()
        })
        .collect();

    let mut checked = 0;
    for key in ["groups", "projects"] {
        let Some(entries) = doc.get(key).and_then(serde_yaml::Value::as_sequence) else {
            continue;
        };
        for entry in entries {
            let Some(ad_groups) = entry
                .get("ad_groups")
                .and_then(serde_yaml::Value::as_sequence)
            else {
                continue;
            };
            for ad_group in ad_groups.iter().filter_map(serde_yaml::Value::as_str) {
                assert!(
                    mapped.iter().any(|m| m == ad_group
                        || systemprompt_web_admin::group_matches_pattern(m, ad_group)),
                    "groups.yaml maps {ad_group}, which no adfs config names in group_roles"
                );
                checked += 1;
            }
        }
    }
    assert!(
        checked > 0,
        "groups.yaml declares no AD mapping at all, which makes this check vacuous"
    );
}
