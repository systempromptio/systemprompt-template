//! Gateway top-level settings (enabled flag, auth scheme, path prefix) and the
//! assembled [`GatewayConfigView`] read view.
//!
//! The read view comes from the loaded services tree rather than from a file
//! read of our own, so what the admin surface reports is what the gateway
//! actually dispatches by, with includes already resolved.

use serde_yaml::Value;
use systemprompt::loader::ServicesBootstrap;
use systemprompt::models::services::{GatewayConfig, GatewayRoute};
use systemprompt_web_shared::error::MarketplaceError;

use crate::types::{GatewayConfigView, GatewayRouteView, UpdateGatewaySettingsRequest};

use std::path::Path;

use super::path::gateway_config_path;
use super::yaml_io::{ensure_gateway_mut, read_gateway_file, write_gateway_file};

pub fn get_gateway_config() -> Result<GatewayConfigView, MarketplaceError> {
    let services = ServicesBootstrap::get()
        // Why: lint-ok: error-adapt — ConfigLoadError is core's variant-less loader error.
        .map_err(|e| MarketplaceError::Internal(format!("services tree is not loaded: {e}")))?;
    let gateway = services.gateway_config().ok_or_else(|| {
        // Why: an absent catalog is an error, never an empty list. The
        // after-the-fact ACL detector iterates these routes, so returning
        // nothing would make it report no violations while checking nothing —
        // a governance surface may not fail open and quiet.
        MarketplaceError::Internal(
            "no gateway configuration in the services tree — expected a `gateway:` block in \
             services/ai/gateway.yaml, included from services/config/config.yaml"
                .to_owned(),
        )
    })?;
    Ok(view_from_gateway(
        gateway,
        &gateway_config_path()?.display().to_string(),
    ))
}

fn view_from_gateway(gateway: &GatewayConfig, config_path: &str) -> GatewayConfigView {
    GatewayConfigView {
        enabled: gateway.enabled,
        auth_scheme: gateway.auth_scheme.clone(),
        inference_path_prefix: gateway.inference_path_prefix.clone(),
        routes: gateway.routes.iter().map(route_view).collect(),
        config_path: config_path.to_owned(),
    }
}

fn route_view(route: &GatewayRoute) -> GatewayRouteView {
    GatewayRouteView {
        id: route.id.as_str().to_owned(),
        model_pattern: route.model_pattern.clone(),
        provider: route.provider.as_str().to_owned(),
        upstream_model: route.upstream_model.clone(),
        extra_headers: route
            .extra_headers
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect(),
        pricing: route
            .pricing
            .as_ref()
            .and_then(|p| serde_yaml::to_value(p).ok()),
        when: route
            .when
            .as_ref()
            .and_then(|w| serde_yaml::to_value(w).ok()),
        requires: route
            .requires
            .as_ref()
            .and_then(|r| serde_yaml::to_value(r).ok()),
    }
}

// Why: an edit has to be readable back immediately, and the loaded services
// tree is a process-wide cell that only reloads on restart. The editor reads
// what it just wrote from the file; every runtime consumer reads the loaded
// tree through `get_gateway_config`.
pub fn get_gateway_config_from_file(path: &Path) -> Result<GatewayConfigView, MarketplaceError> {
    let doc = read_gateway_file(path)?;
    let gateway = doc.get("gateway").ok_or_else(|| {
        MarketplaceError::Internal(format!("{} has no `gateway:` block", path.display()))
    })?;

    let enabled = gateway
        .get("enabled")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let auth_scheme = gateway
        .get("auth_scheme")
        .and_then(Value::as_str)
        .unwrap_or(DEFAULT_AUTH_SCHEME)
        .to_owned();
    let inference_path_prefix = gateway
        .get("inference_path_prefix")
        .and_then(Value::as_str)
        .unwrap_or(DEFAULT_INFERENCE_PATH_PREFIX)
        .to_owned();
    let routes = gateway
        .get("routes")
        .and_then(Value::as_sequence)
        .map(|seq| {
            seq.iter()
                .filter_map(super::yaml_io::route_from_yaml)
                .collect()
        })
        .unwrap_or_default();

    Ok(GatewayConfigView {
        enabled,
        auth_scheme,
        inference_path_prefix,
        routes,
        config_path: path.display().to_string(),
    })
}

const DEFAULT_AUTH_SCHEME: &str = "bearer";
const DEFAULT_INFERENCE_PATH_PREFIX: &str = "/v1";

pub fn update_gateway_settings(
    config_path: &Path,
    req: &UpdateGatewaySettingsRequest,
) -> Result<GatewayConfigView, MarketplaceError> {
    let mut doc = read_gateway_file(config_path)?;
    {
        let gw = ensure_gateway_mut(&mut doc)?;
        if let Some(enabled) = req.enabled {
            gw.insert(Value::from("enabled"), Value::Bool(enabled));
        }
        if let Some(auth_scheme) = &req.auth_scheme {
            gw.insert(Value::from("auth_scheme"), Value::from(auth_scheme.clone()));
        }
        if let Some(prefix) = &req.inference_path_prefix {
            if !prefix.starts_with('/') {
                return Err(MarketplaceError::BadRequest(
                    "inference_path_prefix must start with '/'".into(),
                ));
            }
            gw.insert(
                Value::from("inference_path_prefix"),
                Value::from(prefix.clone()),
            );
        }
    }
    write_gateway_file(config_path, &doc)?;
    get_gateway_config_from_file(config_path)
}
