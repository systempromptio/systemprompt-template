use std::path::PathBuf;

use systemprompt_harness_brand::{Brand, Channel, Identity, identity};

#[test]
fn internal_has_no_channel_label() {
    for brand in Brand::ALL {
        assert_eq!(Identity::of(brand, Channel::Internal).channel_label(), None);
        assert_eq!(
            Identity::of(brand, Channel::Canary).channel_label(),
            Some("Canary")
        );
        assert_eq!(
            Identity::of(brand, Channel::Dev).channel_label(),
            Some("Dev")
        );
    }
}

#[test]
fn brand_tables_hold_the_agreed_values() {
    let sp = Identity::of(Brand::Systemprompt, Channel::Internal);
    assert_eq!(sp.display_name(), "systemprompt Harness");
    assert_eq!(sp.vendor(), "systemprompt");
    assert_eq!(sp.scheme(), "sp-harness");
    assert_eq!(sp.default_instance(), "https://systemprompt.io");
    assert_eq!(sp.app_id(), "io.systemprompt.harness");
}

#[test]
fn data_root_is_vendor_then_display_name() {
    let id = Identity::of(Brand::Systemprompt, Channel::Canary);
    let root = id.data_root();
    assert!(root.ends_with(PathBuf::from("systemprompt").join("systemprompt Harness Canary")));
}

#[test]
fn device_label_names_the_build_and_host() {
    let id = Identity::of(Brand::Systemprompt, Channel::Dev);
    assert_eq!(
        id.device_label("WS-042"),
        "systemprompt Harness Dev on WS-042"
    );
}

#[test]
fn current_identity_is_in_the_table() {
    let id = identity();
    assert_eq!(id, Identity::of(Brand::CURRENT, Channel::CURRENT));
    assert_eq!(id.version(), "0.1.0");
    assert_eq!(id.config_dir_name(), ".harness");
}
