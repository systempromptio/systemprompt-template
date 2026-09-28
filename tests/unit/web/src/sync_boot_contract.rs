//! The one boot contract every sync plane meets: seed an empty projection,
//! otherwise look and report, and refuse to start on a declaration that
//! cannot be read.
//!
//! Pinned against a fake plane so the rule is tested on its own, with no
//! database: the loop only ever calls `drift` and, on an empty projection,
//! `apply` once as the boot actor in overwrite mode.

use std::sync::Mutex;

use async_trait::async_trait;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use systemprompt_web_admin::error::AdminResult;
use systemprompt_web_admin::repositories::sync::boot::{BOOT_ORDER, reconcile_plane};
use systemprompt_web_admin::repositories::sync::plane::{
    Actor, ApplyOutcome, Export, PlaneDrift, SyncMode, SyncPlane,
};
use systemprompt_web_admin::repositories::sync::state::BOOT_ACTOR;

#[derive(Debug, Clone)]
struct Call {
    mode: SyncMode,
    actor: String,
}

struct FakePlane {
    drift: PlaneDrift,
    applied: Mutex<Vec<Call>>,
}

impl FakePlane {
    fn with(drift: PlaneDrift) -> Self {
        Self {
            drift,
            applied: Mutex::new(Vec::new()),
        }
    }

    fn calls(&self) -> Vec<Call> {
        self.applied.lock().expect("calls").clone()
    }
}

#[async_trait]
impl SyncPlane for FakePlane {
    fn id(&self) -> &'static str {
        "fake"
    }

    fn label(&self) -> &'static str {
        "Fake"
    }

    fn source_file(&self) -> &'static str {
        "fake/declaration.yaml"
    }

    fn projection(&self) -> &'static str {
        "fake_rows"
    }

    async fn drift(&self, _pool: &PgPool) -> AdminResult<PlaneDrift> {
        Ok(self.drift.clone())
    }

    async fn apply(
        &self,
        _pool: &PgPool,
        mode: SyncMode,
        actor: Actor<'_>,
    ) -> AdminResult<ApplyOutcome> {
        self.applied.lock().expect("calls").push(Call {
            mode,
            actor: actor.as_str().to_owned(),
        });
        Ok(ApplyOutcome {
            inserted: self.drift.declared_count,
            ..ApplyOutcome::default()
        })
    }

    async fn export(&self, _pool: &PgPool) -> AdminResult<Option<Export>> {
        Ok(None)
    }
}

// Why: the loop never touches the pool itself; a lazy pool satisfies the
// signature without a server.
fn pool() -> PgPool {
    PgPoolOptions::new()
        .connect_lazy("postgres://test:test@localhost:1/test")
        .expect("lazy pool")
}

#[tokio::test]
async fn an_empty_projection_is_seeded_once_as_the_boot_actor() {
    let plane = FakePlane::with(PlaneDrift {
        declared_count: 3,
        in_db: 0,
        is_clean: false,
        ..PlaneDrift::default()
    });
    let boot = reconcile_plane(&pool(), &plane).await.expect("boot");
    assert!(boot.seeded);
    assert_eq!(boot.inserted, 3);
    assert_eq!(boot.drift_rows, 0);
    let calls = plane.calls();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].mode, SyncMode::Overwrite);
    assert_eq!(calls[0].actor, BOOT_ACTOR);
}

#[tokio::test]
async fn a_populated_projection_is_only_compared() {
    let plane = FakePlane::with(PlaneDrift {
        declared_count: 3,
        in_db: 2,
        is_clean: false,
        rows: vec![],
        ..PlaneDrift::default()
    });
    let boot = reconcile_plane(&pool(), &plane).await.expect("boot");
    assert!(!boot.seeded, "a populated projection is never re-seeded");
    assert!(plane.calls().is_empty(), "nothing is written on a restart");
    assert_eq!(boot.in_db, 2);
}

#[tokio::test]
async fn an_empty_declaration_seeds_nothing() {
    let plane = FakePlane::with(PlaneDrift {
        declared_count: 0,
        in_db: 0,
        is_clean: true,
        ..PlaneDrift::default()
    });
    let boot = reconcile_plane(&pool(), &plane).await.expect("boot");
    assert!(!boot.seeded);
    assert!(plane.calls().is_empty());
}

#[tokio::test]
async fn an_unreadable_declaration_fails_the_boot() {
    let plane = FakePlane::with(PlaneDrift {
        in_db: 5,
        unreadable: Some("services tree could not be composed: dangling include".to_owned()),
        ..PlaneDrift::default()
    });
    let err = reconcile_plane(&pool(), &plane)
        .await
        .expect_err("boot refuses");
    let message = err.to_string();
    assert!(message.contains("fake/declaration.yaml"), "{message}");
    assert!(message.contains("dangling include"), "{message}");
    assert!(plane.calls().is_empty());
}

// Why: rules name group and project ids, so groups must exist before the
// rules are seeded; planes added later go after these three.
#[test]
fn boot_order_starts_with_groups_then_access_control_then_gateway_policies() {
    assert!(
        BOOT_ORDER.starts_with(&["groups", "access_control", "gateway_policies"]),
        "{BOOT_ORDER:?}"
    );
}
