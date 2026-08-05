//! The `MarketplaceId` / `RequestId` newtypes are transparent wrappers: every
//! access path (`as_str`, `into_inner`, `Display`, `Deref`, the `From` impls)
//! must yield the same string that went in, so wrapping a value never changes
//! what a query or a template sees.

use std::str::FromStr;
use systemprompt_web_shared::{MarketplaceId, RankTier, RequestId, TierLevel};

#[test]
fn marketplace_id_round_trips_through_every_accessor() {
    let id = MarketplaceId::new("astound-commons");
    assert_eq!(id.as_str(), "astound-commons");
    assert_eq!(id.to_string(), "astound-commons");
    assert_eq!(AsRef::<str>::as_ref(&id), "astound-commons");
    // Deref to `str` means str methods apply directly to the newtype.
    assert!(id.starts_with("astound"));
    assert_eq!(id.into_inner(), "astound-commons");
}

#[test]
fn marketplace_id_from_owned_and_borrowed_are_equal() {
    let from_str = MarketplaceId::from("plugin-a");
    let from_string = MarketplaceId::from("plugin-a".to_owned());
    assert_eq!(from_str, from_string);
    assert_eq!(MarketplaceId::new("plugin-a"), from_str);
}

#[test]
fn request_id_round_trips_through_every_accessor() {
    let id = RequestId::new("req_0123456789");
    assert_eq!(id.as_str(), "req_0123456789");
    assert_eq!(id.to_string(), "req_0123456789");
    assert_eq!(id.len(), "req_0123456789".len());
    assert_eq!(id.clone().into_inner(), "req_0123456789");
    assert_eq!(RequestId::from("req_0123456789"), id);
}

#[test]
fn empty_ids_are_carried_verbatim() {
    // The newtypes do not validate; an empty id stays empty rather than
    // becoming a placeholder.
    assert_eq!(MarketplaceId::new("").as_str(), "");
    assert_eq!(RequestId::new(String::new()).to_string(), "");
}

#[test]
fn rank_tier_round_trips_through_its_wire_string() {
    for tier in [
        RankTier::Bronze,
        RankTier::Silver,
        RankTier::Gold,
        RankTier::Platinum,
        RankTier::Diamond,
    ] {
        assert_eq!(RankTier::from_str(tier.as_str()).unwrap(), tier);
        assert_eq!(tier.to_string(), tier.as_str());
    }
}

#[test]
fn rank_tier_parsing_is_case_insensitive() {
    assert_eq!(RankTier::from_str("GOLD").unwrap(), RankTier::Gold);
    assert_eq!(RankTier::from_str("Diamond").unwrap(), RankTier::Diamond);
}

#[test]
fn rank_tier_rejects_unknown_input_and_names_it() {
    let err = RankTier::from_str("mithril").unwrap_err();
    assert!(
        err.contains("mithril"),
        "error should quote the input: {err}"
    );
}

#[test]
fn rank_tier_defaults_to_bronze() {
    assert_eq!(RankTier::default(), RankTier::Bronze);
}

#[test]
fn tier_level_round_trips_despite_capitalised_display() {
    // `as_str` is title-cased for display, but `FromStr` lower-cases first, so
    // the round trip still closes.
    for tier in [
        TierLevel::Free,
        TierLevel::Pro,
        TierLevel::Team,
        TierLevel::Enterprise,
    ] {
        assert_eq!(TierLevel::from_str(tier.as_str()).unwrap(), tier);
        assert_eq!(tier.to_string(), tier.as_str());
    }
    assert_eq!(TierLevel::Enterprise.as_str(), "Enterprise");
    assert_eq!(TierLevel::from_str("pro").unwrap(), TierLevel::Pro);
}

#[test]
fn tier_level_rejects_unknown_input_and_defaults_to_free() {
    let err = TierLevel::from_str("platinum").unwrap_err();
    assert!(
        err.contains("platinum"),
        "error should quote the input: {err}"
    );
    assert_eq!(TierLevel::default(), TierLevel::Free);
}
