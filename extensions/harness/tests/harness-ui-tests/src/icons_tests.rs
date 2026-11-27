use harness_ui::IconName;

#[test]
fn every_icon_resolves_through_its_asset_path() {
    for icon in IconName::ALL {
        assert_eq!(IconName::from_asset_path(&icon.asset_path()), Some(icon));
    }
}

#[test]
fn every_icon_is_a_24px_stroke_svg() {
    for icon in IconName::ALL {
        let Ok(text) = std::str::from_utf8(icon.svg()) else {
            panic!("{} is not UTF-8", icon.stem());
        };
        assert!(text.contains("viewBox=\"0 0 24 24\""), "{}", icon.stem());
        assert!(text.contains("stroke=\"currentColor\""), "{}", icon.stem());
    }
}

#[test]
fn foreign_paths_are_not_claimed() {
    assert_eq!(IconName::from_asset_path("icons/check.svg"), None);
    assert_eq!(IconName::from_asset_path("harness/icons/unknown.svg"), None);
}
