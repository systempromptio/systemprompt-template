//! Gateway top-level settings (enabled flag, auth scheme, path prefix) and the
//! assembled [`GatewayConfigView`] read view.

use std::path::Path;

use serde_yaml::Value;
use systemprompt_web_shared::error::MarketplaceError;

use crate::types::{GatewayConfigView, UpdateGatewaySettingsRequest};

use super::routes::ensure_route_ids;
use super::yaml_io::{ensure_gateway_mut, read_profile, route_from_yaml, write_profile};

const DEFAULT_AUTH_SCHEME: &str = "bearer";
const DEFAULT_INFERENCE_PATH_PREFIX: &str = "/v1";

pub fn get_gateway_config(profile_path: &Path) -> Result<GatewayConfigView, MarketplaceError> {
    ensure_route_ids(profile_path)?;
    let doc = read_profile(profile_path)?;
    let gateway = doc.get("gateway");

    let enabled = gateway
        .and_then(|g| g.get("enabled"))
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let auth_scheme = gateway
        .and_then(|g| g.get("auth_scheme"))
        .and_then(Value::as_str)
        .unwrap_or(DEFAULT_AUTH_SCHEME)
        .to_owned();
    let inference_path_prefix = gateway
        .and_then(|g| g.get("inference_path_prefix"))
        .and_then(Value::as_str)
        .unwrap_or(DEFAULT_INFERENCE_PATH_PREFIX)
        .to_owned();
    let routes = gateway
        .and_then(|g| g.get("routes"))
        .and_then(Value::as_sequence)
        .map(|seq| seq.iter().filter_map(route_from_yaml).collect())
        .unwrap_or_default();

    Ok(GatewayConfigView {
        enabled,
        auth_scheme,
        inference_path_prefix,
        routes,
        profile_path: profile_path.display().to_string(),
    })
}

pub fn update_gateway_settings(
    profile_path: &Path,
    req: &UpdateGatewaySettingsRequest,
) -> Result<GatewayConfigView, MarketplaceError> {
    let mut doc = read_profile(profile_path)?;
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
    write_profile(profile_path, &doc)?;
    get_gateway_config(profile_path)
}
