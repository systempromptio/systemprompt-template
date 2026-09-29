//! The connection-readiness gate behind the bridge manifest, the `connector`
//! authorization band that must agree with it, and the session-attested
//! `systemprompt` connector an admin holds and a plain user does not.

use std::sync::Arc;

use systemprompt::database::Database;
use systemprompt::identifiers::UserId;
use systemprompt::marketplace::{MarketplaceCandidate, MarketplaceFilter};
use systemprompt::models::bridge::manifest::ManagedMcpServer;
use systemprompt_security::authz::{AccessControlRepository, EntityKind};
use systemprompt_web_admin::authz::connector::connector_rule_type;
use systemprompt_web_admin::authz::subject_attributes_for;
use systemprompt_web_admin::marketplace_filter::TemplateMarketplaceFilter;
use systemprompt_web_admin::repositories::users::connector_accounts as accounts;
use systemprompt_web_admin::test_support::{Connection, NotReady, get_connections};

use crate::fixtures::{insert_user_with_roles, unique};
use crate::tempdb::TempDb;

const CRM: &str = "crm-mcp";
const CONTROL_PLANE: &str = "systemprompt";

// Why: one connector-backed external server and the session-attested control
// plane, installed before the shared `{}` fixture can claim the process-wide
// registry. nextest runs every test in its own process, so the order holds.
fn install_services() {
    if systemprompt::loader::ServicesBootstrap::is_initialized() {
        return;
    }
    let directory = tempfile::tempdir().expect("services fixture");
    let path = directory.path().join("config.yaml");
    std::fs::write(
        &path,
        "mcp_servers:\n\
         \x20 crm-mcp:\n\
         \x20   type: external\n\
         \x20   endpoint: https://crm.example.test/mcp\n\
         \x20   enabled: true\n\
         \x20   display_in_web: false\n\
         \x20   tool_policy: allow\n\
         \x20   oauth: {required: false, scopes: [user], audience: mcp, client_id: null}\n\
         \x20   connector: {adapter: generic, scopes: [tools:read]}\n\
         \x20 systemprompt:\n\
         \x20   type: internal\n\
         \x20   binary: systemprompt-mcp-agent\n\
         \x20   port: 5010\n\
         \x20   enabled: true\n\
         \x20   display_in_web: true\n\
         \x20   tool_policy: allow\n\
         \x20   oauth: {required: true, scopes: [admin], audience: mcp, client_id: null}\n",
    )
    .expect("write services fixture");
    systemprompt::loader::ServicesBootstrap::init_from_path(&path).expect("install services");
    std::mem::forget(directory);
}

struct Harness {
    db: TempDb,
    filter: Arc<dyn MarketplaceFilter>,
}

impl Harness {
    async fn create_or_skip() -> Option<Self> {
        install_services();
        let db = TempDb::create().await?;
        let database: systemprompt::database::DbPool = Arc::new(Database::from_pools(
            Arc::clone(&db.pool),
            Some(Arc::clone(&db.pool)),
        ));
        let filter = TemplateMarketplaceFilter::from_db(&database).expect("build filter");
        let repo = AccessControlRepository::from_pool(Arc::clone(&db.pool));
        for id in [CRM, CONTROL_PLANE] {
            repo.upsert_entity(EntityKind::McpServer, id, true, "test")
                .await
                .expect("upsert entity");
        }
        Some(Self { db, filter })
    }

    async fn user(&self, roles: &[&str]) -> UserId {
        let id = unique("ready-user");
        let roles: Vec<String> = roles.iter().map(|r| (*r).to_owned()).collect();
        insert_user_with_roles(&self.db.pool, &id, &roles).await;
        UserId::new(id)
    }

    async fn record(&self, user: &UserId, provider: &str, verified: bool) {
        let mut tx = self.db.pool.begin().await.expect("begin");
        let mut row = accounts::get_locked_account(&mut tx, user, provider)
            .await
            .expect("lock account");
        row.status = "connected".into();
        row.auth_method = Some("oauth".into());
        row.verified_at = verified.then(chrono::Utc::now);
        accounts::update_account(&mut tx, user, &row)
            .await
            .expect("update account");
        tx.commit().await.expect("commit");
    }

    async fn manifest_servers(&self, user: &UserId) -> (Vec<String>, Vec<String>) {
        let kept = self
            .filter
            .filter(
                user,
                MarketplaceCandidate {
                    managed_mcp_servers: [CRM, CONTROL_PLANE].map(mcp_entry).to_vec(),
                    ..MarketplaceCandidate::default()
                },
            )
            .await
            .expect("filter");
        let servers = kept
            .managed_mcp_servers
            .iter()
            .map(|m| m.id.to_string())
            .collect();
        (servers, kept.diagnostics)
    }

    async fn connection(&self, user: &UserId, provider: &str) -> Connection {
        get_connections(&self.db.pool, user)
            .await
            .expect("connections")
            .connections
            .into_iter()
            .find(|c| c.provider == provider)
            .unwrap_or_else(|| panic!("{provider} is listed"))
    }

    async fn connector_band(&self, user: &UserId) -> Vec<String> {
        subject_attributes_for(&self.db.pool, user)
            .await
            .expect("subject attributes")
            .values(&connector_rule_type())
            .to_vec()
    }
}

fn mcp_entry(name: &str) -> ManagedMcpServer {
    serde_json::from_value(serde_json::json!({
        "name": name, "url": "https://example.test/mcp",
    }))
    .expect("mcp entry")
}

#[tokio::test]
async fn a_connected_and_verified_connector_backed_server_survives_into_the_manifest() {
    let Some(h) = Harness::create_or_skip().await else {
        return;
    };
    let user = h.user(&["user"]).await;
    h.record(&user, CRM, true).await;

    let connection = h.connection(&user, CRM).await;
    assert_eq!(connection.readiness(), Ok(()));
    let (servers, diagnostics) = h.manifest_servers(&user).await;
    assert!(servers.contains(&CRM.to_owned()), "kept {servers:?}");
    assert!(
        !diagnostics.iter().any(|d| d.contains(CRM)),
        "a ready server raises no diagnostic: {diagnostics:?}"
    );
    assert!(
        h.connector_band(&user).await.contains(&CRM.to_owned()),
        "the authz band admits the same server the manifest carries"
    );

    h.db.cleanup().await;
}

#[tokio::test]
async fn an_admitted_but_unverified_server_is_dropped_and_says_why() {
    let Some(h) = Harness::create_or_skip().await else {
        return;
    };
    let user = h.user(&["user"]).await;
    h.record(&user, CRM, false).await;

    let connection = h.connection(&user, CRM).await;
    assert_eq!(connection.status, "connected");
    assert_eq!(connection.readiness(), Err(NotReady::Unverified));
    let (servers, diagnostics) = h.manifest_servers(&user).await;
    assert!(!servers.contains(&CRM.to_owned()), "kept {servers:?}");
    let diagnostic = diagnostics
        .iter()
        .find(|d| d.contains(CRM))
        .unwrap_or_else(|| panic!("the drop is recorded: {diagnostics:?}"));
    assert!(
        diagnostic.contains("verified_at_null") && diagnostic.contains("status=connected"),
        "the diagnostic names the failing sub-condition: {diagnostic}"
    );
    assert!(
        !h.connector_band(&user).await.contains(&CRM.to_owned()),
        "a `connected` row without verification opens no rule either"
    );

    h.db.cleanup().await;
}

#[tokio::test]
async fn the_control_plane_is_a_connected_connector_for_an_admin_only() {
    let Some(h) = Harness::create_or_skip().await else {
        return;
    };
    let admin = h.user(&["admin"]).await;
    let plain = h.user(&["user"]).await;

    let for_admin = h.connection(&admin, CONTROL_PLANE).await;
    assert!(for_admin.requires_auth && for_admin.session_attested);
    assert!(for_admin.entitled);
    assert_eq!(for_admin.status, "connected");
    assert!(for_admin.verified_at.is_some());
    assert_eq!(for_admin.actions, ["test"]);
    assert_eq!(for_admin.readiness(), Ok(()));

    let for_plain = h.connection(&plain, CONTROL_PLANE).await;
    assert!(for_plain.requires_auth && for_plain.session_attested);
    assert!(!for_plain.entitled);
    assert_ne!(for_plain.status, "no_auth_required");
    assert!(for_plain.actions.is_empty());
    assert_eq!(for_plain.readiness(), Err(NotReady::NotEntitled));

    let (admin_servers, _) = h.manifest_servers(&admin).await;
    let (plain_servers, plain_diagnostics) = h.manifest_servers(&plain).await;
    assert!(admin_servers.contains(&CONTROL_PLANE.to_owned()));
    assert!(!plain_servers.contains(&CONTROL_PLANE.to_owned()));
    assert!(
        plain_diagnostics
            .iter()
            .any(|d| d.contains(CONTROL_PLANE) && d.contains("not_entitled")),
        "{plain_diagnostics:?}"
    );
    assert!(
        h.connector_band(&admin)
            .await
            .contains(&CONTROL_PLANE.to_owned())
    );
    assert!(
        !h.connector_band(&plain)
            .await
            .contains(&CONTROL_PLANE.to_owned())
    );

    h.db.cleanup().await;
}
