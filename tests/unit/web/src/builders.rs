//! The creation-parameter builders exist so a caller never positionally fills
//! a dozen optional columns. Each one must start from a defined default state
//! and let every optional field be set independently — a `with_*` that
//! silently dropped its argument would write a null column with no error.

use chrono::{TimeZone, Utc};
use systemprompt::identifiers::{
    CampaignId, CategoryId, ContentId, ContextId, LinkClickId, LinkId, SessionId, SourceId, TaskId,
    UserId,
};
use systemprompt_web_shared::models::{
    ContentKind, ContentLinkMetadata, ContentSeed, CreateContentParams, CreateLinkParams,
    RecordClickParams, TrackClickParams,
};

fn link_metadata(title: &str) -> ContentLinkMetadata {
    ContentLinkMetadata {
        title: title.to_owned(),
        url: format!("https://astounddigital.com/{title}"),
    }
}

fn seed() -> ContentSeed {
    ContentSeed {
        slug: "governance-101".to_owned(),
        title: "Governance 101".to_owned(),
        description: "How the spine works".to_owned(),
        body: "# Governance".to_owned(),
        author: "Ed".to_owned(),
        published_at: Utc.with_ymd_and_hms(2026, 1, 2, 3, 4, 5).unwrap(),
        source_id: SourceId::new("guides"),
    }
}

#[test]
fn create_link_params_defaults_to_an_active_link_with_no_campaign() {
    let params = CreateLinkParams::new(
        "abc123".to_owned(),
        "https://astounddigital.com/pricing".to_owned(),
        "redirect",
    );
    assert_eq!(params.short_code, "abc123");
    assert_eq!(params.target_url, "https://astounddigital.com/pricing");
    assert_eq!(params.link_type, "redirect");
    assert!(params.is_active, "a new link is live unless disabled");
    assert!(params.source_content_id.is_none());
    assert!(params.source_page.is_none());
    assert!(params.campaign_id.is_none());
    assert!(params.campaign_name.is_none());
    assert!(params.utm_params.is_none());
    assert!(params.link_text.is_none());
    assert!(params.link_position.is_none());
    assert!(params.destination_type.is_none());
    assert!(params.expires_at.is_none());
}

#[test]
fn create_link_params_full_chain_sets_every_field() {
    let expires = Utc.with_ymd_and_hms(2027, 6, 1, 0, 0, 0).unwrap();
    let params = CreateLinkParams::new(
        "abc123".to_owned(),
        "https://astounddigital.com/pricing".to_owned(),
        "utm",
    )
    .with_source_content_id(Some(ContentId::new("content_1")))
    .with_source_page(Some("/blog/governance".to_owned()))
    .with_campaign_id(Some(CampaignId::new("camp_1")))
    .with_campaign_name(Some("Q3 launch".to_owned()))
    .with_utm_params(Some("{\"source\":\"linkedin\"}".to_owned()))
    .with_link_text(Some("See pricing".to_owned()))
    .with_link_position(Some("footer".to_owned()))
    .with_destination_type(Some("internal".to_owned()))
    .with_is_active(false)
    .with_expires_at(Some(expires));

    assert_eq!(
        params.source_content_id.as_ref().map(ContentId::as_str),
        Some("content_1")
    );
    assert_eq!(params.source_page.as_deref(), Some("/blog/governance"));
    assert_eq!(
        params.campaign_id.as_ref().map(CampaignId::as_str),
        Some("camp_1")
    );
    assert_eq!(params.campaign_name.as_deref(), Some("Q3 launch"));
    assert_eq!(
        params.utm_params.as_deref(),
        Some("{\"source\":\"linkedin\"}")
    );
    assert_eq!(params.link_text.as_deref(), Some("See pricing"));
    assert_eq!(params.link_position.as_deref(), Some("footer"));
    assert_eq!(params.destination_type.as_deref(), Some("internal"));
    assert!(!params.is_active);
    assert_eq!(params.expires_at, Some(expires));
}

#[test]
fn create_content_params_defaults_carry_the_seed_and_zero_the_rest() {
    let params = CreateContentParams::new(seed());
    assert_eq!(params.slug, "governance-101");
    assert_eq!(params.title, "Governance 101");
    assert_eq!(params.description, "How the spine works");
    assert_eq!(params.body, "# Governance");
    assert_eq!(params.author, "Ed");
    assert_eq!(params.source_id.as_str(), "guides");
    assert_eq!(
        params.published_at.timestamp(),
        seed().published_at.timestamp()
    );

    assert_eq!(params.kind, ContentKind::Blog);
    assert!(params.keywords.is_empty());
    assert!(params.version_hash.is_empty());
    assert!(params.image.is_none());
    assert!(params.category_id.is_none());
    assert!(params.category.is_none());
    assert!(params.links.0.is_empty());
    assert!(params.after_reading_this.0.is_empty());
    assert!(params.related_playbooks.0.is_empty());
    assert!(params.related_code.0.is_empty());
    assert!(params.related_docs.0.is_empty());
}

#[test]
fn create_content_params_full_chain_sets_every_field() {
    let params = CreateContentParams::new(seed())
        .with_keywords("governance, mcp".to_owned())
        .with_kind(ContentKind::Guide)
        .with_image(Some("/img/hero.png".to_owned()))
        .with_category_id(Some(CategoryId::new("cat_1")))
        .with_category(Some("guides".to_owned()))
        .with_version_hash("deadbeef".to_owned())
        .with_links(vec![link_metadata("a")])
        .with_after_reading_this(vec!["Read the spec".to_owned()])
        .with_related_playbooks(vec![link_metadata("b")])
        .with_related_code(vec![link_metadata("c")])
        .with_related_docs(vec![link_metadata("d"), link_metadata("e")]);

    assert_eq!(params.keywords, "governance, mcp");
    assert_eq!(params.kind, ContentKind::Guide);
    assert_eq!(params.image.as_deref(), Some("/img/hero.png"));
    assert_eq!(
        params.category_id.as_ref().map(CategoryId::as_str),
        Some("cat_1")
    );
    assert_eq!(params.category.as_deref(), Some("guides"));
    assert_eq!(params.version_hash, "deadbeef");
    assert_eq!(params.links.0[0].title, "a");
    assert_eq!(
        params.after_reading_this.0,
        vec!["Read the spec".to_owned()]
    );
    assert_eq!(params.related_playbooks.0[0].title, "b");
    assert_eq!(params.related_code.0[0].title, "c");
    assert_eq!(params.related_docs.0.len(), 2);
}

#[test]
fn record_click_params_defaults_to_a_repeat_non_converting_click() {
    let clicked_at = Utc.with_ymd_and_hms(2026, 5, 5, 12, 0, 0).unwrap();
    let params = RecordClickParams::new(
        LinkClickId::new("click_1"),
        LinkId::new("link_1"),
        SessionId::new("sess_1"),
        clicked_at,
    );
    assert_eq!(params.click_id.as_str(), "click_1");
    assert_eq!(params.link_id.as_str(), "link_1");
    assert_eq!(params.session_id.as_str(), "sess_1");
    assert_eq!(params.clicked_at, clicked_at);
    assert!(!params.is_first_click);
    assert!(!params.is_conversion);
    assert!(params.user_id.is_none());
    assert!(params.context_id.is_none());
    assert!(params.task_id.is_none());
    assert!(params.referrer_page.is_none());
    assert!(params.referrer_url.is_none());
    assert!(params.user_agent.is_none());
    assert!(params.ip_address.is_none());
    assert!(params.device_type.is_none());
    assert!(params.country.is_none());
}

#[test]
fn record_click_params_full_chain_sets_every_field() {
    let params = RecordClickParams::new(
        LinkClickId::new("click_1"),
        LinkId::new("link_1"),
        SessionId::new("sess_1"),
        Utc.with_ymd_and_hms(2026, 5, 5, 12, 0, 0).unwrap(),
    )
    .with_user_id(Some(UserId::new("user_1")))
    .with_context_id(Some(ContextId::new_unchecked("9b1deb4d-3b7d-4bad-9bdd-2b0d7b3dcb6d")))
    .with_task_id(Some(TaskId::new("task_1")))
    .with_referrer_page(Some("/blog".to_owned()))
    .with_referrer_url(Some("https://news.example/x".to_owned()))
    .with_user_agent(Some("curl/8".to_owned()))
    .with_ip_address(Some("203.0.113.7".to_owned()))
    .with_device_type(Some("desktop".to_owned()))
    .with_country(Some("ES".to_owned()))
    .with_is_first_click(true)
    .with_is_conversion(true);

    assert_eq!(params.user_id.as_ref().map(UserId::as_str), Some("user_1"));
    assert_eq!(
        params.context_id.as_ref().map(ContextId::as_str),
        Some("9b1deb4d-3b7d-4bad-9bdd-2b0d7b3dcb6d")
    );
    assert_eq!(params.task_id.as_ref().map(TaskId::as_str), Some("task_1"));
    assert_eq!(params.referrer_page.as_deref(), Some("/blog"));
    assert_eq!(
        params.referrer_url.as_deref(),
        Some("https://news.example/x")
    );
    assert_eq!(params.user_agent.as_deref(), Some("curl/8"));
    assert_eq!(params.ip_address.as_deref(), Some("203.0.113.7"));
    assert_eq!(params.device_type.as_deref(), Some("desktop"));
    assert_eq!(params.country.as_deref(), Some("ES"));
    assert!(params.is_first_click);
    assert!(params.is_conversion);
}

#[test]
fn track_click_params_defaults_hold_only_the_link_and_session() {
    let params = TrackClickParams::new(LinkId::new("link_1"), SessionId::new("sess_1"));
    assert_eq!(params.link_id.as_str(), "link_1");
    assert_eq!(params.session_id.as_str(), "sess_1");
    assert!(params.user_id.is_none());
    assert!(params.context_id.is_none());
    assert!(params.task_id.is_none());
    assert!(params.referrer_page.is_none());
    assert!(params.referrer_url.is_none());
    assert!(params.user_agent.is_none());
    assert!(params.ip_address.is_none());
    assert!(params.device_type.is_none());
    assert!(params.country.is_none());
}

#[test]
fn track_click_params_full_chain_sets_every_field() {
    let params = TrackClickParams::new(LinkId::new("link_1"), SessionId::new("sess_1"))
        .with_user_id(Some(UserId::new("user_1")))
        .with_context_id(Some(ContextId::new_unchecked("9b1deb4d-3b7d-4bad-9bdd-2b0d7b3dcb6d")))
        .with_task_id(Some(TaskId::new("task_1")))
        .with_referrer_page(Some("/pricing".to_owned()))
        .with_referrer_url(Some("https://ref.example".to_owned()))
        .with_user_agent(Some("Mozilla/5.0".to_owned()))
        .with_ip_address(Some("198.51.100.4".to_owned()))
        .with_device_type(Some("mobile".to_owned()))
        .with_country(Some("GB".to_owned()));

    assert_eq!(params.user_id.as_ref().map(UserId::as_str), Some("user_1"));
    assert_eq!(
        params.context_id.as_ref().map(ContextId::as_str),
        Some("9b1deb4d-3b7d-4bad-9bdd-2b0d7b3dcb6d")
    );
    assert_eq!(params.task_id.as_ref().map(TaskId::as_str), Some("task_1"));
    assert_eq!(params.referrer_page.as_deref(), Some("/pricing"));
    assert_eq!(params.referrer_url.as_deref(), Some("https://ref.example"));
    assert_eq!(params.user_agent.as_deref(), Some("Mozilla/5.0"));
    assert_eq!(params.ip_address.as_deref(), Some("198.51.100.4"));
    assert_eq!(params.device_type.as_deref(), Some("mobile"));
    assert_eq!(params.country.as_deref(), Some("GB"));
}
