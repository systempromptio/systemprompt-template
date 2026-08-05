//! `UpdateContentParams` is built from a required seed plus optional layers,
//! and the update statement writes every field it carries. An un-layered
//! build must therefore leave `image` and `links` as `None` — the repository
//! reads `None` as "leave the stored value alone", so a builder that
//! defaulted to `Some(empty)` would silently wipe a page's image and link
//! metadata on every ingest.

use systemprompt::identifiers::ContentId;
use systemprompt_web_extension::models::ContentLinkMetadata;
use systemprompt_web_extension::repository::{UpdateContentParams, UpdateContentSeed};

fn seed() -> UpdateContentSeed {
    UpdateContentSeed {
        id: ContentId::new("content-1"),
        title: "Title".to_owned(),
        description: "Description".to_owned(),
        body: "Body".to_owned(),
        keywords: "a,b".to_owned(),
        version_hash: "hash-1".to_owned(),
    }
}

#[test]
fn a_bare_build_carries_the_seed_and_no_optional_fields() {
    let params = UpdateContentParams::builder(seed()).build();
    assert_eq!(params.id.as_str(), "content-1");
    assert_eq!(params.title, "Title");
    assert_eq!(params.description, "Description");
    assert_eq!(params.body, "Body");
    assert_eq!(params.keywords, "a,b");
    assert_eq!(params.version_hash, "hash-1");
    assert!(params.image.is_none());
    assert!(params.links.is_none());
}

#[test]
fn the_optional_layers_chain_in_either_order() {
    let params = UpdateContentParams::builder(seed())
        .with_image("/img/hero.png")
        .with_links(vec![ContentLinkMetadata {
            title: "Docs".to_owned(),
            url: "https://a.test/docs".to_owned(),
        }])
        .build();
    assert_eq!(params.image.as_deref(), Some("/img/hero.png"));
    let links = params.links.expect("links were layered on");
    assert_eq!(links.0.len(), 1);
    assert_eq!(links.0[0].url, "https://a.test/docs");
}

#[test]
fn an_empty_link_list_is_still_a_write() {
    let params = UpdateContentParams::builder(seed())
        .with_links(Vec::new())
        .build();
    let links = params
        .links
        .expect("an explicit empty list clears the links");
    assert!(links.0.is_empty());
}

#[test]
fn the_last_image_layered_on_wins() {
    let params = UpdateContentParams::builder(seed())
        .with_image("/img/old.png")
        .with_image("/img/new.png")
        .build();
    assert_eq!(params.image.as_deref(), Some("/img/new.png"));
}
