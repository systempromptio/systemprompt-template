//! Owner-scoped evaluator enrollment. Runtime inputs come from managed
//! publications.

use super::evaluation_experiments::require_write_origin;
use crate::error::{AdminError, AdminResult};
use crate::routes::evaluation_state::EvaluationState;
use crate::types::UserContext;
use axum::extract::Path;
use axum::http::{HeaderMap, StatusCode};
use axum::{Extension, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use systemprompt::identifiers::EvalWorkerId;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct EnrollWorker {
    name: String,
}

#[derive(Serialize)]
pub(crate) struct Enrollment {
    id: EvalWorkerId,
    token: String,
}

pub(crate) async fn enroll(
    Extension(_user): Extension<UserContext>,
    Extension(state): Extension<Arc<EvaluationState>>,
    headers: HeaderMap,
    Json(input): Json<EnrollWorker>,
) -> AdminResult<impl axum::response::IntoResponse> {
    require_write_origin(&headers)?;
    let profile = systemprompt::config::ProfileBootstrap::get().map_err(AdminError::internal)?;
    let credential = state
        .workers
        .create(&state.owner, &profile.server.api_external_url, &input.name)
        .await?;
    let token = credential.expose_token().to_owned();
    Ok((
        StatusCode::CREATED,
        [("cache-control", "no-store")],
        Json(Enrollment {
            id: credential.id,
            token,
        }),
    ))
}

pub(crate) async fn revoke(
    Extension(_user): Extension<UserContext>,
    Extension(state): Extension<Arc<EvaluationState>>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> AdminResult<StatusCode> {
    require_write_origin(&headers)?;
    state
        .workers
        .revoke(&state.owner, &EvalWorkerId::new(id))
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
