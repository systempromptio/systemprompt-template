//! `POST /admin/auth/adfs/acs` — the assertion consumer service. Verify
//! the posted `SAMLResponse`, gate its claims, resolve the identity to a local
//! user, and set the session.

use std::sync::LazyLock;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use axum::Extension;
use axum::extract::Form;
use axum::extract::rejection::FormRejection;
use axum::http::HeaderMap;
use axum::http::header::SET_COOKIE;
use axum::response::{IntoResponse, Redirect, Response};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use saml::sp::LoginTracker;
use saml::{
    ConsumeResponse, Identity, InMemoryReplayCache, ReplayMode, SsoResponseBinding,
    SsoResponseEndpoint,
};
use serde::Deserialize;

use super::identity::{AssertionClaims, resolve_identity};
use super::provider::{idp_descriptor, service_provider};
use super::session::{SessionSubject, mint_session, session_cookie};
use super::{
    AdfsDeps, AdfsError, FlowState, clear_state_cookie, login_error, read_state_cookie,
    sanitize_redirect, use_https,
};

// Why: an assertion presented twice inside its validity window is a replay,
// whatever else is right about it. In-process is enough for a single
// instance; a multi-instance deploy would move this to Postgres.
static REPLAY_CACHE: LazyLock<InMemoryReplayCache> = LazyLock::new(InMemoryReplayCache::default);

#[derive(Deserialize)]
pub(crate) struct CallbackForm {
    #[serde(rename = "SAMLResponse")]
    saml_response: String,
    #[serde(rename = "RelayState")]
    relay_state: Option<String>,
}

struct SuccessfulLogin {
    redirect_to: String,
    jwt: String,
    max_age: i64,
}

pub(crate) async fn adfs_callback(
    Extension(deps): Extension<AdfsDeps>,
    headers: HeaderMap,
    form: Result<Form<CallbackForm>, FormRejection>,
) -> Response {
    // Why: lint-ok: http-error — every outcome here is a redirect: success sets the
    // session cookie and returns to the app, failure returns to login.
    if !deps.config.is_usable() {
        return login_error("unavailable");
    }
    let Ok(Form(form)) = form else {
        return login_error("error");
    };
    match run_callback(&deps, &headers, form).await {
        Ok(login) => success_response(&login),
        Err(reason) => login_error(reason),
    }
}

async fn run_callback(
    deps: &AdfsDeps,
    headers: &HeaderMap,
    form: CallbackForm,
) -> Result<SuccessfulLogin, &'static str> {
    let cfg = &deps.config;
    let flow = correlate(
        cfg.allow_idp_initiated,
        headers,
        form.relay_state.as_deref(),
    )?;
    let (idp, identity) = verify_response(deps, &form.saml_response, flow.as_ref())
        .await
        .map_err(|e| {
            tracing::warn!(error = %e, "ADFS assertion rejected");
            match e {
                AdfsError::Assertion(saml::Error::XmlParse(m)) if m.contains("NameID") => {
                    "no_subject"
                },
                // Why: the farm answered an AuthnRequest we started, but we
                // reached the verifier with no tracker — the state cookie did
                // not come back — so the SP had to treat a solicited response
                // as unsolicited. That is a cookie fault, not a bad assertion,
                // and it gets its own reason so it never again reads as one.
                AdfsError::Assertion(saml::Error::UnsolicitedNotAllowed) => "correlation_lost",
                AdfsError::Metadata(_) | AdfsError::ServiceProvider(_) => "unavailable",
                _ => "invalid_assertion",
            }
        })?;

    let claims = AssertionClaims::from_identity(&identity);
    let resolved = resolve_identity(deps, &idp, &claims).await?;

    let (jwt, max_age) = mint_session(
        &deps.session_service,
        &SessionSubject::from(&resolved),
        headers,
    )
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "Failed to mint session for ADFS user");
        "error"
    })?;

    tracing::info!(
        user_id = %resolved.user_id,
        email = %resolved.email,
        roles = ?resolved.roles,
        assertion_id = %identity.assertion_id,
        "ADFS SSO login succeeded"
    );

    Ok(SuccessfulLogin {
        redirect_to: flow.map_or_else(|| sanitize_redirect(None), |f| f.redirect_to),
        jwt,
        max_age,
    })
}

// Why: A solicited response must echo the RelayState the start handler
// minted, and the cookie must be present to compare it against. With no
// cookie at all this is an IdP-initiated sign-in, allowed only when
// configured; a cookie whose nonce disagrees is a forged or replayed return.
fn correlate(
    allow_idp_initiated: bool,
    headers: &HeaderMap,
    relay_state: Option<&str>,
) -> Result<Option<FlowState>, &'static str> {
    match read_state_cookie(headers) {
        Some(flow) => {
            if relay_state != Some(flow.state.as_str()) {
                tracing::warn!("ADFS RelayState does not match the state cookie");
                return Err("error");
            }
            Ok(Some(flow))
        },
        None if allow_idp_initiated => Ok(None),
        None => {
            tracing::warn!(
                "ADFS response arrived with no state cookie and IdP-initiated sign-in is off"
            );
            Err("error")
        },
    }
}

async fn verify_response(
    deps: &AdfsDeps,
    saml_response_b64: &str,
    flow: Option<&FlowState>,
) -> Result<(String, Identity), AdfsError> {
    let cfg = &deps.config;
    let idp = idp_descriptor(cfg).await?;
    let sp = service_provider(cfg)?;
    let xml = STANDARD
        .decode(saml_response_b64.trim())
        .map_err(|e| AdfsError::Base64(e.to_string()))?;

    let tracker = flow.map(|f| LoginTracker {
        request_id: f.request_id.clone(),
        issued_at: UNIX_EPOCH + Duration::from_secs(f.issued_at_unix),
        idp_entity_id: idp.entity_id.clone(),
        acs_endpoint: SsoResponseEndpoint::post(cfg.acs_url.clone(), 0, true),
        requested_authn_context: None,
        requested_name_id_format: None,
    });

    let identity = sp
        .consume_response(ConsumeResponse {
            idp: &idp,
            peer_crypto_policy: None,
            saml_response: &xml,
            binding: SsoResponseBinding::HttpPost,
            relay_state: flow.map(|f| f.state.as_str()),
            tracker: tracker.as_ref(),
            expected_destination: &cfg.acs_url,
            now: SystemTime::now(),
            clock_skew: Duration::from_secs(cfg.clock_skew_seconds),
            replay_cache: Some(&*REPLAY_CACHE),
            replay_mode: ReplayMode::All,
            holder_of_key_cert: None,
        })
        .map_err(AdfsError::Assertion)?;
    Ok((idp.entity_id.clone(), identity))
}

/// Build the cookie-setting redirect for a successful login: clear the spent
/// state cookie and set the session `access_token`.
// Why: lint-ok: http-error — builds the success redirect, not an error.
fn success_response(login: &SuccessfulLogin) -> Response {
    let mut out = HeaderMap::new();
    if let Ok(val) = clear_state_cookie(use_https()).parse() {
        out.append(SET_COOKIE, val);
    }
    if let Ok(val) = session_cookie(&login.jwt, login.max_age).parse() {
        out.append(SET_COOKIE, val);
    }
    (out, Redirect::to(&login.redirect_to)).into_response()
}
