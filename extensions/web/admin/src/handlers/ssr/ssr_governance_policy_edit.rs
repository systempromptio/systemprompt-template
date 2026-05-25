//! `/admin/governance/{policy_id}` — per-policy detail / editor.
//!
//! Reads the live `Policy` instance for `policy_id` from the inventory
//! registry and pairs it with the recent decisions that policy has produced.
//! The editor surface is intentionally informational + a single
//! enable/disable toggle (`POST`ed to the same path); deeper parameter edits
//! still happen in `services/governance/config.yaml` because YAML is the
//! source of truth and we want operators looking at the file when they
//! change limits.

use std::sync::Arc;

use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect, Response},
    Form,
};
use serde::Deserialize;
use serde_json::json;
use sqlx::PgPool;

use crate::handlers::webhook::governance;
use crate::repositories;
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext, DECISION_DENY};

use super::ACCESS_DENIED_HTML;

const RECENT_LIMIT: i64 = 50;

pub async fn governance_policy_edit_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Path(policy_id): Path<String>,
) -> Response {
    if !user_ctx.is_admin {
        return (StatusCode::FORBIDDEN, Html(ACCESS_DENIED_HTML)).into_response();
    }

    let snapshot: Option<(String, String, String, String, bool, String)> = {
        let chain = governance::chain();
        let result = chain
            .iter()
            .find(|(_, p)| p.id().as_str() == policy_id)
            .map(|(cfg, p)| {
                let id = p.id().as_str().to_string();
                (
                    id.clone(),
                    p.name().to_string(),
                    p.description().to_string(),
                    serde_yaml::to_string(&cfg.params).unwrap_or_default(),
                    cfg.enabled,
                    id,
                )
            });
        result
    };

    let Some((id_str, name, description, params_yaml, enabled, lookup_id)) = snapshot else {
        let data = json!({
            "page": "governance",
            "title": "Unknown policy",
            "policy_id": policy_id,
        });
        return super::render_page(
            &engine,
            "governance-unknown-policy",
            &data,
            &user_ctx,
            &mkt_ctx,
        );
    };

    let recent = repositories::governance::list_decisions_for_policy(
        &pool,
        &lookup_id,
        RECENT_LIMIT,
    )
    .await
    .unwrap_or_else(|e| {
        tracing::warn!(error = %e, policy = %lookup_id, "per-policy decisions query failed");
        Vec::new()
    });

    let recent_json: Vec<serde_json::Value> = recent
        .iter()
        .map(|r| {
            json!({
                "id": r.id,
                "user_id": r.user_id,
                "tool_name": r.tool_name,
                "agent_id": r.agent_id,
                "agent_scope": r.agent_scope,
                "decision": r.decision,
                "is_denied": r.decision == DECISION_DENY,
                "reason": r.reason,
                "created_at": r.created_at
                    .with_timezone(&chrono::Local)
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string(),
            })
        })
        .collect();

    let data = json!({
        "page": "governance",
        "title": format!("{name} — Policy"),
        "policy": {
            "id": id_str,
            "name": name,
            "description": description,
            "enabled": enabled,
        },
        "params_yaml": params_yaml,
        "recent": recent_json,
        "has_recent": !recent_json.is_empty(),
        "config_path": "services/governance/config.yaml",
    });

    super::render_page(
        &engine,
        "governance-policy-edit",
        &data,
        &user_ctx,
        &mkt_ctx,
    )
}

#[derive(Debug, Deserialize)]
pub struct ToggleForm {
    pub enabled: Option<String>,
}

/// POST /admin/governance/{id}/toggle — flip `enabled` on the named policy in
/// `services/governance/config.yaml`, then ask the registry to re-read so
/// the change takes effect without a process restart.
pub async fn governance_policy_toggle(
    Extension(user_ctx): Extension<UserContext>,
    Path(policy_id): Path<String>,
    Form(form): Form<ToggleForm>,
) -> Response {
    if !user_ctx.is_admin {
        return (StatusCode::FORBIDDEN, Html(ACCESS_DENIED_HTML)).into_response();
    }

    let want_enabled = form
        .enabled
        .as_deref()
        .is_some_and(|v| matches!(v, "true" | "on" | "1"));

    if let Err(e) = update_enabled_in_yaml(&policy_id, want_enabled) {
        tracing::error!(error = %e, policy = %policy_id, "failed to update governance config");
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Html(format!(
                "<p>Could not update services/governance/config.yaml: {e}</p>"
            )),
        )
            .into_response();
    }

    governance::reload();
    Redirect::to(&format!("/admin/governance/{policy_id}")).into_response()
}

fn update_enabled_in_yaml(policy_id: &str, enabled: bool) -> Result<(), String> {
    use systemprompt::config::ProfileBootstrap;
    let bootstrap = ProfileBootstrap::get().map_err(|e| e.to_string())?;
    let path = std::path::PathBuf::from(&bootstrap.paths.services).join("governance/config.yaml");

    let text =
        std::fs::read_to_string(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
    let mut root: serde_yaml::Value =
        serde_yaml::from_str(&text).map_err(|e| format!("parse YAML: {e}"))?;

    let policies = root
        .get_mut("governance")
        .and_then(|g| g.get_mut("policies"))
        .and_then(|p| p.as_sequence_mut())
        .ok_or_else(|| "governance.policies missing or not a sequence".to_string())?;

    let mut found = false;
    for entry in policies.iter_mut() {
        let matches = entry
            .get("id")
            .and_then(|v| v.as_str())
            .is_some_and(|s| s == policy_id);
        if matches {
            if let serde_yaml::Value::Mapping(map) = entry {
                map.insert(
                    serde_yaml::Value::String("enabled".to_string()),
                    serde_yaml::Value::Bool(enabled),
                );
                found = true;
                break;
            }
        }
    }

    if !found {
        let mut map = serde_yaml::Mapping::new();
        map.insert(
            serde_yaml::Value::String("id".to_string()),
            serde_yaml::Value::String(policy_id.to_string()),
        );
        map.insert(
            serde_yaml::Value::String("enabled".to_string()),
            serde_yaml::Value::Bool(enabled),
        );
        policies.push(serde_yaml::Value::Mapping(map));
    }

    let updated = serde_yaml::to_string(&root).map_err(|e| format!("serialize: {e}"))?;
    std::fs::write(&path, updated).map_err(|e| format!("write {}: {e}", path.display()))?;
    Ok(())
}
