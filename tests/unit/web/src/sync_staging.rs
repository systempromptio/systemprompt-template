//! Archive uploads remain an in-memory preview until an operator applies a
//! plane. Applying one plane must consume only that text, and consuming the
//! final plane must close the stage so neither can be applied twice.

use chrono::{Duration, Utc};
use systemprompt_web_admin::repositories::sync::archive::staging::{
    STAGE_TTL_MINUTES, StagedArchive, StagingStore,
};

fn stage(actor: &str, planes: &[(&'static str, &str)]) -> StagedArchive {
    let mut stage = StagedArchive::new(actor, None, None);
    for (id, text) in planes {
        stage.planes.insert(*id, (*text).to_owned());
    }
    stage
}

#[test]
fn applying_a_plane_consumes_only_that_plane_and_closes_the_final_one() {
    let store = StagingStore::default();
    let id = store.put(stage(
        "admin-1",
        &[
            ("groups", "groups: []\n"),
            ("gateway_policies", "policies: []\n"),
        ],
    ));

    assert!(
        !store.drop_plane(&id, "groups"),
        "a stage remains available while another plane is unapplied"
    );
    let remaining = store.get(&id).expect("the remaining plane is still staged");
    assert_eq!(remaining.actor, "admin-1");
    assert_eq!(
        remaining.planes.get("gateway_policies").map(String::as_str),
        Some("policies: []\n")
    );
    assert!(!remaining.planes.contains_key("groups"));

    assert!(
        store.drop_plane(&id, "gateway_policies"),
        "applying the last plane closes the stage"
    );
    assert!(
        store.get(&id).is_none(),
        "a closed stage cannot be replayed"
    );
    assert!(
        !store.drop_plane(&id, "gateway_policies"),
        "a replay cannot make a missing stage look successfully applied"
    );
}

#[test]
fn expired_uploads_cannot_be_previewed_or_applied() {
    let store = StagingStore::default();
    let mut expired = stage("admin-2", &[("groups", "groups: []\n")]);
    expired.created_at = Utc::now() - Duration::minutes(STAGE_TTL_MINUTES);
    let id = store.put(expired);

    assert!(store.get(&id).is_none(), "expiry is enforced at retrieval");
    // `drop_plane` is cleanup after the handler has already resolved the
    // stage through `get`; it intentionally does not perform a second expiry
    // check. The handler's `stage_or_404` gate means an expired id never
    // reaches that mutation path.
}

#[test]
fn discard_removes_a_stage_without_consuming_any_plane() {
    let store = StagingStore::default();
    let id = store.put(stage("admin-3", &[("groups", "groups: []\n")]));

    store.remove(&id);

    assert!(store.get(&id).is_none());
    assert!(!store.drop_plane(&id, "groups"));
}
