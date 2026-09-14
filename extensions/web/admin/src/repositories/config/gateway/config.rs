//! Gateway top-level settings (enabled flag, auth scheme, path prefix) and the
//! assembled [`GatewayConfigView`] read view.

use std::path::Path;

use serde_yaml::Value;
use systemprompt_web_shared::error::MarketplaceError;

use crate::types::{GatewayConfigView, UpdateGatewaySettingsRequest};

use super::yaml_io::{ensure_gateway_mut, read_gateway_file, route_from_yaml, write_gateway_file};

const DEFAULT_AUTH_SCHEME: &str = "bearer";
const DEFAULT_INFERENCE_PATH_PREFIX: &str = "/v1";

// Why: a read never writes. This used to backfill synthesized route ids into
// the file, so merely opening the gateway page rewrote it — dropping its
// comments and adding fields nobody typed. Ids missing from the file are
// synthesized in memory by `route_from_yaml`, deterministically, which is all
// the callers ever needed.
pub fn get_gateway_config(gateway_path: &Path) -> Result<GatewayConfigView, MarketplaceError> {
    let doc = read_gateway_file(gateway_path)?;
    // Why: an absent catalog is an error, never an empty list. Every surface
    // built on this view reports routes it found; a silent zero would read as
    // a deployment with no gateway rather than a file the editor cannot see.
    let gateway = doc.get("gateway").ok_or_else(|| {
        MarketplaceError::Internal(format!(
            "{} has no `gateway:` block",
            gateway_path.display()
        ))
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
        .map(|seq| seq.iter().filter_map(route_from_yaml).collect())
        .unwrap_or_default();

    Ok(GatewayConfigView {
        enabled,
        auth_scheme,
        inference_path_prefix,
        routes,
        source_path: gateway_path.display().to_string(),
    })
}

pub fn update_gateway_settings(
    gateway_path: &Path,
    req: &UpdateGatewaySettingsRequest,
) -> Result<GatewayConfigView, MarketplaceError> {
    let mut doc = read_gateway_file(gateway_path)?;
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
    write_gateway_file(gateway_path, &doc)?;
    get_gateway_config(gateway_path)
}
