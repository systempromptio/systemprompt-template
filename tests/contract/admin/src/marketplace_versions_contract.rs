//! Marketplace version comparison is an operator-facing audit: manifests
//! must retain the exact add/remove/change picture, and untrusted manifest
//! identifiers must be rendered as text.

use axum::http::StatusCode;
use serde_json::json;

use crate::app::{App, Call};
use crate::principal::Principal;
use crate::tempdb::TempDb;
use crate::{globals, principal, seed};

async fn record_version(
    pool: &sqlx::PgPool,
    marketplace: &str,
    hash: &str,
    manifest: serde_json::Value,
    is_current: bool,
) {
    sqlx::query(
        "INSERT INTO marketplace_versions
             (marketplace_id, content_hash, source, manifest, plugin_count, skill_count,
              first_seen_at, last_seen_at, effective_until)
         VALUES ($1, $2, 'contract fixture', $3::jsonb, 1, 3,
                 CASE WHEN $4 THEN now() ELSE now() - interval '2 days' END,
                 now(),
                 CASE WHEN $4 THEN NULL ELSE now() - interval '1 day' END)",
    )
    .bind(marketplace)
    .bind(hash)
    .bind(manifest.to_string())
    .bind(is_current)
    .execute(pool)
    .await
    .expect("record marketplace version");
}

fn manifest(marketplace: &str, version: &str, skills: serde_json::Value) -> serde_json::Value {
    json!({
        "marketplace_id": marketplace,
        "name": "Contract version marketplace",
        "version": version,
        "plugins": [{
            "plugin_id": "contract-version-plugin",
            "digest": format!("plugin-{version}"),
            "skills": skills,
        }],
        "files": 3,
    })
}

fn skill_row<'a>(body: &'a str, skill_id: &str) -> &'a str {
    let skill_at = body.find(skill_id).expect("skill is rendered");
    let row_start = body[..skill_at].rfind("<tr>").expect("skill row starts");
    let row_end = skill_at + body[skill_at..].find("</tr>").expect("skill row ends");
    &body[row_start..row_end]
}

#[tokio::test(flavor = "multi_thread")]
async fn comparison_renders_manifest_changes_and_escapes_skill_identifiers() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);
    let marketplace = seed::unique("version-marketplace");
    let older_hash = "a".repeat(64);
    let newer_hash = "b".repeat(64);
    let changed = seed::unique("version-changed");
    let removed = seed::unique("version-removed");
    let added = seed::unique("version-added");
    let hostile = "<script>alert(1)</script>";

    record_version(
        &db.pool,
        &marketplace,
        &older_hash,
        manifest(
            &marketplace,
            "1.0.0",
            json!([
                {"skill_id": changed, "skill_key": "changed", "digest": "old"},
                {"skill_id": removed, "skill_key": "removed", "digest": "removed"},
            ]),
        ),
        false,
    )
    .await;
    record_version(
        &db.pool,
        &marketplace,
        &newer_hash,
        manifest(
            &marketplace,
            "2.0.0",
            json!([
                {"skill_id": changed, "skill_key": "changed", "digest": "new"},
                {"skill_id": added, "skill_key": "added", "digest": "added"},
                {"skill_id": hostile, "skill_key": "untrusted", "digest": "hostile"},
            ]),
        ),
        true,
    )
    .await;

    let path =
        format!("/admin/analysis/versions/{marketplace}?tab=compare&a={older_hash}&b={newer_hash}");
    let (status, body) = app.call(Call::get(&path, Principal::Admin)).await;
    assert_eq!(status, StatusCode::OK, "compare page: {body}");
    assert!(
        body.contains("title=\"2 added · 1 removed · 1 changed · 0 unchanged\""),
        "comparison totals distinguish every manifest movement: {body}"
    );
    assert!(skill_row(&body, &changed).contains(">changed<"));
    assert!(skill_row(&body, &removed).contains(">removed<"));
    assert!(skill_row(&body, &added).contains(">added<"));
    let escaped_hostile = "&lt;script&gt;alert(1)&lt;/script&gt;";
    assert!(
        skill_row(&body, escaped_hostile).contains(escaped_hostile),
        "untrusted identifier is text, not markup: {body}"
    );
    assert!(
        !body.contains(hostile),
        "raw script tag must not be rendered"
    );

    let missing_hash = "c".repeat(64);
    let missing_path = format!(
        "/admin/analysis/versions/{marketplace}?tab=compare&a={missing_hash}&b={newer_hash}"
    );
    let (status, body) = app.call(Call::get(&missing_path, Principal::Admin)).await;
    assert_eq!(status, StatusCode::NOT_FOUND, "missing version: {body}");
    db.cleanup().await;
}
