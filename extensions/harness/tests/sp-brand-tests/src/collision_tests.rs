use std::collections::HashSet;

use sp_brand::Identity;

fn all() -> Vec<&'static Identity> {
    Identity::ALL
        .iter()
        .map(|&(brand, channel)| Identity::of(brand, channel))
        .collect()
}

#[test]
fn table_rows_match_their_keys() {
    for &(brand, channel) in &Identity::ALL {
        let id = Identity::of(brand, channel);
        assert_eq!(id.brand(), brand);
        assert_eq!(id.channel(), channel);
        assert!(id.app_id().ends_with(channel.app_id_suffix()));
        assert!(id.scheme().ends_with(channel.scheme_suffix()));
        assert_eq!(
            id.display_name(),
            format!("{}{}", brand.product(), channel.display_suffix())
        );
    }
}

#[test]
fn no_two_builds_share_an_app_id() {
    let ids: HashSet<_> = all().iter().map(|id| id.app_id()).collect();
    assert_eq!(ids.len(), Identity::ALL.len());
}

#[test]
fn no_two_builds_share_a_scheme() {
    let schemes: HashSet<_> = all().iter().map(|id| id.scheme()).collect();
    assert_eq!(schemes.len(), Identity::ALL.len());
}

#[test]
fn no_two_builds_share_a_data_root() {
    let roots: HashSet<_> = all().iter().map(|id| id.data_root()).collect();
    assert_eq!(roots.len(), Identity::ALL.len());
    let relative: HashSet<_> = all().iter().map(|id| id.data_dir_relative()).collect();
    assert_eq!(relative.len(), Identity::ALL.len());
}

#[test]
fn no_two_builds_share_a_display_name() {
    let names: HashSet<_> = all().iter().map(|id| id.display_name()).collect();
    assert_eq!(names.len(), Identity::ALL.len());
}

#[test]
fn schemes_are_valid_url_schemes() {
    for id in all() {
        let scheme = id.scheme();
        assert!(scheme.starts_with(|c: char| c.is_ascii_lowercase()));
        assert!(scheme.chars().all(|c| {
            c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '+' | '-' | '.')
        }));
    }
}
