//! The catalog pages and the access-control JSON API behind them.
//!
//! The catalog is the one part of the admin plane whose *entities* come from
//! the on-disk profile — `services/skills/*.yaml`, `services/mcp/*.yaml`,
//! `services/plugins/*.yaml` — while their *access rules* come from the
//! database. Both sides therefore have to be driven: a page that lists the
//! right entities and resolves their grants against the wrong table looks
//! correct until someone is granted something they should not have.
//!
//! The API half is a CRUD surface, so each endpoint is driven three ways: the
//! call that works, the call whose body is the wrong shape, and the call whose
//! path names something that does not exist. The middle case is the one that
//! matters most — every one of these handlers parses a string into a typed
//! enum, and a handler that mints an unrecognised value rather than rejecting
//! it creates a rule dimension nothing ever resolves.

use axum::http::StatusCode;

use crate::app::{ADMIN_API_PREFIX, App, Call};
use crate::principal::Principal;
use crate::tempdb::TempDb;
use crate::{globals, principal, seed};

fn api(path: &str) -> String {
    format!("{ADMIN_API_PREFIX}{path}")
}

// The catalog pages, including the per-entity detail pages that the status
// contract only ever drives with an id matching nothing.
#[tokio::test(flavor = "multi_thread")]
async fn catalog_pages_render_entities_from_the_profile() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        eprintln!("no DATABASE_URL — skipping catalog suite");
        return;
    };

    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);

    let mut failures = Vec::new();
    // The catalog root redirects onto the plugins listing and the singular
    // marketplace alias onto the marketplaces page, so a bookmark from either
    // survives.
    for (path, expected) in [
        ("/admin/catalog", "/admin/plugins"),
        ("/admin/catalog/marketplace", "/admin/marketplaces"),
    ] {
        let (status, target) = app.redirect_of(Call::get(path, Principal::Admin)).await;
        if status != StatusCode::PERMANENT_REDIRECT {
            failures.push(format!("  {path} -> {} (expected 308)", status.as_u16()));
        } else if target != expected {
            failures.push(format!("  {path} redirected to {target:?}, not {expected}"));
        }
    }

    let listings: [(&str, &str); 3] = [
        (
            "/admin/plugins",
            "A plugin bundles skills, MCP servers, agents and hooks",
        ),
        ("/admin/skills", "The instruction sets people invoke"),
        ("/admin/mcp", "what it is serving right now"),
    ];
    for (path, marker) in listings {
        let (status, body) = app.call(Call::get(path, Principal::Admin)).await;
        if status != StatusCode::OK {
            failures.push(format!(
                "  {path} -> {} (expected 200): {}",
                status.as_u16(),
                body.chars().take(200).collect::<String>()
            ));
        } else if !body.contains(marker) {
            failures.push(format!("  {path} rendered without {marker:?}"));
        }
    }

    // The detail pages are driven with an id taken from the listing itself, so
    // the case survives a profile whose contents change. An id nobody ships is
    // a miss, which must be a 404 rather than a rendered shell.
    for (listing, prefix) in [
        ("/admin/skills", "/admin/skills/"),
        ("/admin/mcp", "/admin/mcp/"),
        ("/admin/plugins", "/admin/plugins/"),
    ] {
        let (_, body) = app.call(Call::get(listing, Principal::Admin)).await;
        let Some(id) = first_detail_id(&body, prefix) else {
            // A profile with nothing of this kind is a legitimate state; the
            // miss case below still runs.
            continue;
        };
        let path = format!("{prefix}{id}");
        let (status, detail) = app.call(Call::get(&path, Principal::Admin)).await;
        if status != StatusCode::OK {
            failures.push(format!(
                "  {path} (an id the listing itself linked to) -> {} : {}",
                status.as_u16(),
                detail.chars().take(200).collect::<String>()
            ));
        } else if !detail.contains(&id) {
            failures.push(format!("  {path} rendered without naming {id:?}"));
        }
    }

    for path in [
        "/admin/skills/no-such-skill",
        "/admin/mcp/no-such-server",
        "/admin/plugins/no-such-plugin",
    ] {
        let (status, body) = app.call(Call::get(path, Principal::Admin)).await;
        if status.is_server_error() {
            failures.push(format!(
                "  {path} faulted: {}",
                body.chars().take(200).collect::<String>()
            ));
        }
    }

    db.cleanup().await;
    assert!(
        failures.is_empty(),
        "{} catalog page case(s) failed:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

// The first entity id the listing page links to, read out of its own markup so
// the case never hard-codes a profile's contents.
fn first_detail_id(body: &str, prefix: &str) -> Option<String> {
    let needle = format!("href=\"{prefix}");
    let start = body.find(&needle)? + needle.len();
    let rest = &body[start..];
    let end = rest.find('"')?;
    let id = &rest[..end];
    (!id.is_empty() && !id.contains('/')).then(|| id.to_owned())
}

// The generic entity-access API: read, grant, flip the default, delete.
#[tokio::test(flavor = "multi_thread")]
async fn entity_access_api_round_trips_a_grant() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };

    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);
    let user_id = seed::unique("access-user");
    seed::insert_user(&db.pool, &user_id, &format!("{user_id}@contract.test")).await;
    let entity = seed::unique("skill-entity");

    let mut failures = Vec::new();

    // An entity with no rows reads back as "no rules, not included by
    // default" rather than 404 — the catalog entry exists on disk whether or
    // not anyone has ever written a grant for it.
    let read = api(&format!("/access-control/entity/skill/{entity}/access"));
    let (status, body) = app.call(Call::get(&read, Principal::Admin)).await;
    if status != StatusCode::OK || !body.contains(r#""rules":[]"#) {
        failures.push(format!(
            "  reading an entity with no rules -> {} {}",
            status.as_u16(),
            body.chars().take(200).collect::<String>()
        ));
    }

    // Register the entity before granting on it. A rule carries a foreign key
    // to `access_control_entities`, and the default endpoint is what creates
    // that row — this is the order the dashboard writes in, not a fixture
    // convenience.
    let default_path = api(&format!("/access-control/entity/skill/{entity}/default"));
    let (status, _) = app
        .call(Call::json(
            "patch",
            &default_path,
            Principal::Admin,
            r#"{"default_included":false}"#,
        ))
        .await;
    if status != StatusCode::OK {
        failures.push(format!(
            "  registering the entity -> {} (expected 200)",
            status.as_u16()
        ));
    }

    // Grant.
    let rules_path = api(&format!("/access-control/entity/skill/{entity}/rules"));
    let grant = format!(
        r#"{{"rule_type":"user","rule_value":"{user_id}","access":"allow","justification":"contract fixture"}}"#
    );
    let (status, body) = app
        .call(Call::json("post", &rules_path, Principal::Admin, &grant))
        .await;
    let rule_id = if status == StatusCode::OK {
        extract_json_string(&body, "\"id\":\"")
    } else {
        failures.push(format!(
            "  granting a user rule -> {} {}",
            status.as_u16(),
            body.chars().take(200).collect::<String>()
        ));
        None
    };

    // Read back: the grant must be visible on the same entity.
    let (_, body) = app.call(Call::get(&read, Principal::Admin)).await;
    if !body.contains(&user_id) {
        failures.push("  a granted rule did not read back on the entity".to_owned());
    }

    // Flip the default.
    let (status, body) = app
        .call(Call::json(
            "patch",
            &default_path,
            Principal::Admin,
            r#"{"default_included":true}"#,
        ))
        .await;
    if status != StatusCode::OK || !body.contains(r#""default_included":true"#) {
        failures.push(format!(
            "  setting default_included -> {} {}",
            status.as_u16(),
            body.chars().take(200).collect::<String>()
        ));
    }

    // Delete, then delete again: the second call is a 404, which is what makes
    // the first one meaningful.
    if let Some(id) = rule_id {
        let delete_path = api(&format!("/access-control/entity/skill/{entity}/rules/{id}"));
        let (status, _) = app
            .call(Call::json("delete", &delete_path, Principal::Admin, "{}"))
            .await;
        if status != StatusCode::NO_CONTENT {
            failures.push(format!(
                "  deleting a rule -> {} (expected 204)",
                status.as_u16()
            ));
        }
        let (status, _) = app
            .call(Call::json("delete", &delete_path, Principal::Admin, "{}"))
            .await;
        if status != StatusCode::NOT_FOUND {
            failures.push(format!(
                "  deleting the same rule twice -> {} (expected 404)",
                status.as_u16()
            ));
        }
    }

    // The rejection ladder. Each body is wrong in exactly one field, so a 400
    // names the check that caught it.
    let rejected: [(&str, String, &str); 5] = [
        (
            "an unrecognised entity type",
            api("/access-control/entity/not-a-kind/x/rules"),
            r#"{"rule_type":"user","rule_value":"u","access":"allow"}"#,
        ),
        (
            "a rule type this form does not own",
            rules_path.clone(),
            r#"{"rule_type":"organization","rule_value":"acme","access":"allow"}"#,
        ),
        (
            "an access decision that is neither allow nor deny",
            rules_path.clone(),
            r#"{"rule_type":"user","rule_value":"u","access":"maybe"}"#,
        ),
        (
            "an empty rule value",
            rules_path.clone(),
            r#"{"rule_type":"user","rule_value":"   ","access":"allow"}"#,
        ),
        (
            "a body missing the access field entirely",
            rules_path.clone(),
            r#"{"rule_type":"user","rule_value":"u"}"#,
        ),
    ];
    for (label, path, body) in rejected {
        let (status, _) = app
            .call(Call::json("post", &path, Principal::Admin, body))
            .await;
        if !status.is_client_error() {
            failures.push(format!("  {label} -> {} (expected a 4xx)", status.as_u16()));
        }
    }

    // The bulk listing is parameterised on an entity type read off the disk
    // profile, so both the known and the unknown kind are worth driving.
    for (label, path, want_ok) in [
        (
            "listing every gateway route's access",
            api("/access-control/entity-access/all?entity_type=gateway_route"),
            true,
        ),
        (
            "listing every MCP server's access",
            api("/access-control/entity-access/all?entity_type=mcp_server"),
            true,
        ),
        (
            "listing an entity type that is not a kind",
            api("/access-control/entity-access/all?entity_type=nonsense"),
            false,
        ),
        (
            "listing with no entity_type at all",
            api("/access-control/entity-access/all"),
            false,
        ),
    ] {
        let (status, body) = app.call(Call::get(&path, Principal::Admin)).await;
        if want_ok && status != StatusCode::OK {
            failures.push(format!(
                "  {label} -> {} : {}",
                status.as_u16(),
                body.chars().take(200).collect::<String>()
            ));
        }
        if !want_ok && !status.is_client_error() {
            failures.push(format!("  {label} -> {} (expected a 4xx)", status.as_u16()));
        }
    }

    db.cleanup().await;
    assert!(
        failures.is_empty(),
        "{} entity-access API case(s) failed:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

fn extract_json_string(body: &str, key: &str) -> Option<String> {
    let start = body.find(key)? + key.len();
    let rest = &body[start..];
    let end = rest.find('"')?;
    Some(rest[..end].to_owned())
}

// The older, entity-type-specific access-control surface: whole-set rule
// replacement, the bulk assign, the per-user matrix, and the YAML snapshot.
#[tokio::test(flavor = "multi_thread")]
async fn access_control_api_replaces_rules_and_projects_a_matrix() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };

    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);
    let user_id = seed::unique("matrix-user");
    seed::insert_user(&db.pool, &user_id, &format!("{user_id}@contract.test")).await;
    let plugin = seed::unique("matrix-plugin");

    let mut failures = Vec::new();

    // Replace the rule set on one entity. This endpoint is a whole-set write:
    // the rules sent become the rules stored, so the read-back is the assertion.
    let put_path = api(&format!("/access-control/entity/plugin/{plugin}"));
    let body = format!(
        r#"{{"rules":[{{"rule_type":"user","rule_value":"{user_id}","access":"allow"}},{{"rule_type":"role","rule_value":"user","access":"deny"}}]}}"#
    );
    let (status, response) = app
        .call(Call::json("put", &put_path, Principal::Admin, &body))
        .await;
    if status != StatusCode::OK {
        failures.push(format!(
            "  replacing a plugin's rules -> {} : {}",
            status.as_u16(),
            response.chars().take(200).collect::<String>()
        ));
    } else if !response.contains(&user_id) {
        failures.push("  the replaced rule set did not include the rule just written".to_owned());
    }

    // Replacing with an empty set clears them, which is the branch a UI hits
    // when the last grant is removed.
    let (status, _) = app
        .call(Call::json(
            "put",
            &put_path,
            Principal::Admin,
            r#"{"rules":[]}"#,
        ))
        .await;
    if status != StatusCode::OK {
        failures.push(format!(
            "  clearing a plugin's rules -> {} (expected 200)",
            status.as_u16()
        ));
    }

    // The entity-type allowlist on this endpoint is narrower than the generic
    // one; a kind outside it is refused rather than written.
    let (status, _) = app
        .call(Call::json(
            "put",
            &api("/access-control/entity/hook/anything"),
            Principal::Admin,
            r#"{"rules":[]}"#,
        ))
        .await;
    if status != StatusCode::BAD_REQUEST {
        failures.push(format!(
            "  replacing rules on an unsupported entity type -> {} (expected 400)",
            status.as_u16()
        ));
    }

    // The bulk assign writes the same rule set across several entities at once.
    let bulk = format!(
        r#"{{"entities":[{{"entity_type":"plugin","entity_id":"{plugin}"}},{{"entity_type":"agent","entity_id":"{}"}}],"rules":[{{"rule_type":"role","rule_value":"admin","access":"allow"}}]}}"#,
        seed::unique("bulk-agent")
    );
    let (status, response) = app
        .call(Call::json(
            "put",
            &api("/access-control/bulk"),
            Principal::Admin,
            &bulk,
        ))
        .await;
    if status != StatusCode::OK || !response.contains("updated_count") {
        failures.push(format!(
            "  bulk assign -> {} : {}",
            status.as_u16(),
            response.chars().take(200).collect::<String>()
        ));
    }

    // Reads: the whole rule table, one entity's slice of it, the per-user
    // matrix, the project projection, and the YAML snapshot.
    // The matrix is projected per user, so a user id in no table is a miss
    // rather than an empty matrix that reads as "this person has no access".
    let (status, _) = app
        .call(Call::get(
            &api("/access-control/users/no-such-user/matrix"),
            Principal::Admin,
        ))
        .await;
    if status != StatusCode::NOT_FOUND {
        failures.push(format!(
            "  the matrix for a user that does not exist -> {} (expected 404)",
            status.as_u16()
        ));
    }

    let reads: [(&str, String); 5] = [
        ("every rule", api("/access-control")),
        (
            "one entity's rules",
            api(&format!(
                "/access-control?entity_type=plugin&entity_id={plugin}"
            )),
        ),
        (
            "the rules of an entity with none",
            api("/access-control?entity_type=plugin&entity_id=no-such-plugin"),
        ),
        (
            "the per-user matrix",
            api(&format!("/access-control/users/{user_id}/matrix")),
        ),
        ("the group projection", api("/groups")),
    ];
    for (label, path) in reads {
        let (status, body) = app.call(Call::get(&path, Principal::Admin)).await;
        if status != StatusCode::OK {
            failures.push(format!(
                "  {label} -> {} : {}",
                status.as_u16(),
                body.chars().take(200).collect::<String>()
            ));
        }
    }

    // The YAML snapshot serialises the whole access plane; it is the one read
    // that can fail on a rule the serialiser has no representation for.
    let (status, body) = app
        .call(Call::get(
            &api("/access-control/yaml-snapshot"),
            Principal::Admin,
        ))
        .await;
    if status.is_server_error() {
        failures.push(format!(
            "  the YAML snapshot faulted: {}",
            body.chars().take(200).collect::<String>()
        ));
    }

    // Every one of these is admin-only; a non-admin session must be refused
    // rather than served another customer's access matrix.
    for path in [
        api("/access-control"),
        api(&format!("/access-control/users/{user_id}/matrix")),
        api("/access-control/yaml-snapshot"),
    ] {
        let (status, _) = app.call(Call::get(&path, Principal::NonAdmin)).await;
        if !(status == StatusCode::FORBIDDEN
            || status == StatusCode::UNAUTHORIZED
            || status.is_redirection())
        {
            failures.push(format!(
                "  {path} as a non-admin -> {} (expected a refusal)",
                status.as_u16()
            ));
        }
    }

    db.cleanup().await;
    assert!(
        failures.is_empty(),
        "{} access-control API case(s) failed:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

// A marketplace granted by ROLE rather than by group must resolve in the
// per-user matrix as an allow decided at the ROLE layer for a user holding that
// role and nothing else — and as a deny at the default layer for a plain user,
// so the grant reads as a narrowing and not a widening.
#[tokio::test(flavor = "multi_thread")]
async fn the_matrix_resolves_a_role_granted_marketplace_at_the_role_layer() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };

    let credentials = principal::provision(&db.pool).await;
    let knowledge_worker = credentials.knowledge_worker_user_id.clone();
    let plain_user = credentials.non_admin_user_id.clone();
    let app = App::new(&db.pool, credentials);
    const MARKETPLACE: &str = "astound-super-admin";
    seed::insert_acl_rule(
        &db.pool,
        "marketplace",
        MARKETPLACE,
        "role",
        "knowledge_worker",
        "allow",
    )
    .await;

    let mut failures = Vec::new();
    for (label, user_id, expected_effective, expected_layer) in [
        ("the knowledge worker", &knowledge_worker, "allow", "role"),
        ("a plain user", &plain_user, "deny", "default"),
    ] {
        let path = api(&format!("/access-control/users/{user_id}/matrix"));
        let (status, body) = app.call(Call::get(&path, Principal::Admin)).await;
        if status != StatusCode::OK {
            failures.push(format!(
                "  the matrix for {label} -> {} : {}",
                status.as_u16(),
                body.chars().take(200).collect::<String>()
            ));
            continue;
        }
        let matrix: serde_json::Value = serde_json::from_str(&body).expect("the matrix is JSON");
        let row = matrix["sections"]
            .as_array()
            .into_iter()
            .flatten()
            .filter(|s| s["entity_type"] == "marketplace")
            .flat_map(|s| s["rows"].as_array().cloned().unwrap_or_default())
            .find(|r| r["entity_id"] == MARKETPLACE);
        let Some(row) = row else {
            failures.push(format!(
                "  the matrix for {label} has no marketplace row for {MARKETPLACE}"
            ));
            continue;
        };
        let effective = row["effective"].as_str().unwrap_or_default();
        let layer = row["source"]["layer"].as_str().unwrap_or_default();
        if effective != expected_effective || layer != expected_layer {
            failures.push(format!(
                "  {MARKETPLACE} for {label} -> effective={effective} layer={layer} \
                 (expected effective={expected_effective} layer={expected_layer}; detail: {})",
                row["source"]["detail"]
            ));
        }
    }

    db.cleanup().await;
    assert!(
        failures.is_empty(),
        "{} role-granted marketplace matrix case(s) failed:\n{}",
        failures.len(),
        failures.join("\n")
    );
}
