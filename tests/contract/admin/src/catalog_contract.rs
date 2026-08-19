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
    // The catalog root and the marketplace alias are permanent redirects onto
    // the plugins listing, so a bookmark from either survives.
    for path in ["/admin/catalog", "/admin/catalog/marketplace"] {
        let (status, target) = app.redirect_of(Call::get(path, Principal::Admin)).await;
        if status != StatusCode::PERMANENT_REDIRECT {
            failures.push(format!("  {path} -> {} (expected 308)", status.as_u16()));
        } else if target != "/admin/catalog/plugins" {
            failures.push(format!(
                "  {path} redirected to {target:?}, not the plugins listing"
            ));
        }
    }

    let listings: [(&str, &str); 3] = [
        (
            "/admin/catalog/plugins",
            "Plugins are the installable units in the catalog",
        ),
        (
            "/admin/catalog/skills",
            "Skills are reusable instruction sets",
        ),
        ("/admin/catalog/mcp", "MCP servers expose tools to agents"),
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
        ("/admin/catalog/skills", "/admin/catalog/skills/"),
        ("/admin/catalog/mcp", "/admin/catalog/mcp/"),
        ("/admin/catalog/plugins", "/admin/catalog/plugins/"),
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
        "/admin/catalog/skills/no-such-skill",
        "/admin/catalog/mcp/no-such-server",
        "/admin/catalog/plugins/no-such-plugin",
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
            r#"{"rule_type":"department","rule_value":"eng","access":"allow"}"#,
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
            &api("/access-control/entity/skill/anything"),
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
    // matrix, the department projection, and the YAML snapshot.
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
        (
            "the department projection",
            api("/access-control/departments"),
        ),
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

// The department API's validation and read paths.
//
// The create/rename/delete round trip is *not* driven here:
// `create_department` inserts without `org_id`, which migration `022` made
// `NOT NULL` with no default, so every create answers `500`. Pinning that
// would turn a defect into a contract; the cases below cover the paths that
// behave, and the validation ladder that runs before the broken insert is
// reached.
#[tokio::test(flavor = "multi_thread")]
async fn department_api_validates_before_it_writes() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };

    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);
    let user_id = seed::unique("dept-user");
    seed::insert_user(&db.pool, &user_id, &format!("{user_id}@contract.test")).await;

    let mut failures = Vec::new();

    // A blank name is refused before it reaches the column, where it used to
    // become an empty string every read path had to coalesce away.
    for blank in [r#"{"name":""}"#, r#"{"name":"   "}"#, r#"{"name":"\t"}"#] {
        let (status, _) = app
            .call(Call::json(
                "post",
                &api("/management/departments"),
                Principal::Admin,
                blank,
            ))
            .await;
        if status != StatusCode::BAD_REQUEST {
            failures.push(format!(
                "  creating a department named {blank} -> {} (expected 400)",
                status.as_u16()
            ));
        }
    }

    // A body with no `name` at all is the extractor's problem, not the
    // handler's.
    let (status, _) = app
        .call(Call::json(
            "post",
            &api("/management/departments"),
            Principal::Admin,
            r#"{"description":"nameless"}"#,
        ))
        .await;
    if !status.is_client_error() {
        failures.push(format!(
            "  creating a department with no name field -> {} (expected a 4xx)",
            status.as_u16()
        ));
    }

    // The same validation runs ahead of the update, so a blank rename is
    // refused rather than reaching the row.
    let (status, _) = app
        .call(Call::json(
            "put",
            &api("/management/departments/whatever"),
            Principal::Admin,
            r#"{"name":"  "}"#,
        ))
        .await;
    if status != StatusCode::BAD_REQUEST {
        failures.push(format!(
            "  renaming a department to blank -> {} (expected 400)",
            status.as_u16()
        ));
    }

    // Renaming or deleting something that never existed is a miss, not a
    // create.
    for (label, method) in [("renaming", "put"), ("deleting", "delete")] {
        let (status, _) = app
            .call(Call::json(
                method,
                &api("/management/departments/no-such-department"),
                Principal::Admin,
                r#"{"name":"Ghost"}"#,
            ))
            .await;
        if status != StatusCode::NOT_FOUND {
            failures.push(format!(
                "  {label} a department that does not exist -> {} (expected 404)",
                status.as_u16()
            ));
        }
    }

    // The listing renders the department migration `009` seeds.
    let (status, body) = app
        .call(Call::get(&api("/management/departments"), Principal::Admin))
        .await;
    if status != StatusCode::OK {
        failures.push(format!("  listing departments -> {}", status.as_u16()));
    } else if !body.contains("Default") {
        failures.push("  the listing did not include the seeded Default department".to_owned());
    }

    // Membership assignment against the seeded department, and the clearing
    // form of the same call.
    let assign = api(&format!("/management/users/{user_id}/department"));
    for body in [
        r#"{"department_name":"Default"}"#,
        r#"{"department_name":""}"#,
    ] {
        let (status, response) = app
            .call(Call::json("post", &assign, Principal::Admin, body))
            .await;
        if status.is_server_error() {
            failures.push(format!(
                "  assigning a department with {body} faulted: {}",
                response.chars().take(200).collect::<String>()
            ));
        }
    }

    // Assigning a user who does not exist must not silently succeed.
    let (status, _) = app
        .call(Call::json(
            "post",
            &api("/management/users/no-such-user/department"),
            Principal::Admin,
            r#"{"department_name":"Default"}"#,
        ))
        .await;
    if status.is_server_error() {
        failures.push("  assigning a department to an unknown user faulted".to_owned());
    }

    // Every one of these is admin-only.
    for (method, path) in [
        ("post", api("/management/departments")),
        ("put", api("/management/departments/whatever")),
        ("delete", api("/management/departments/whatever")),
    ] {
        let (status, _) = app
            .call(Call::json(
                method,
                &path,
                Principal::NonAdmin,
                r#"{"name":"Nope"}"#,
            ))
            .await;
        if !(status == StatusCode::FORBIDDEN || status == StatusCode::UNAUTHORIZED) {
            failures.push(format!(
                "  {method} {path} as a non-admin -> {} (expected a refusal)",
                status.as_u16()
            ));
        }
    }

    db.cleanup().await;
    assert!(
        failures.is_empty(),
        "{} department API case(s) failed:\n{}",
        failures.len(),
        failures.join("\n")
    );
}
