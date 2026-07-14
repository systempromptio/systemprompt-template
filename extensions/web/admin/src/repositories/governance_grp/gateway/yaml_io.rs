//! Profile YAML read/write and route <-> YAML conversion.
//!
//! All mutation paths funnel through [`read_profile`] / [`write_profile`] so
//! the `gateway` block stays well-formed, and through [`ensure_gateway_mut`] /
//! [`routes_seq_mut`] which lazily create the block and `routes` sequence.

use std::path::Path;

use serde_yaml::{Mapping, Value};
use systemprompt_web_shared::error::MarketplaceError;

use crate::types::GatewayRouteView;

use super::matching::synthesize_route_id;

pub(super) fn read_profile(profile_path: &Path) -> Result<Value, MarketplaceError> {
    let content = std::fs::read_to_string(profile_path)?;
    let doc: Value = serde_yaml::from_str(&content)?;
    Ok(doc)
}

pub(super) fn write_profile(profile_path: &Path, doc: &Value) -> Result<(), MarketplaceError> {
    let yaml_str = serde_yaml::to_string(doc)?;
    std::fs::write(profile_path, yaml_str).map_err(|e| {
        MarketplaceError::Internal(format!(
            "Failed to write profile {}: {e}",
            profile_path.display()
        ))
    })?;
    Ok(())
}

pub(super) fn route_from_yaml(val: &Value) -> Option<GatewayRouteView> {
    let map = val.as_mapping()?;
    let model_pattern = map.get(Value::from("model_pattern"))?.as_str()?.to_owned();
    let provider = map.get(Value::from("provider"))?.as_str()?.to_owned();
    let upstream_model = map
        .get(Value::from("upstream_model"))
        .and_then(Value::as_str)
        .map(str::to_owned);
    let extra_headers = map
        .get(Value::from("extra_headers"))
        .and_then(Value::as_mapping)
        .map(|m| {
            m.iter()
                .filter_map(|(k, v)| Some((k.as_str()?.to_owned(), v.as_str()?.to_owned())))
                .collect()
        })
        .unwrap_or_default();
    let id = map
        .get(Value::from("id"))
        .and_then(Value::as_str)
        .map_or_else(
            || synthesize_route_id(&model_pattern, &provider),
            str::to_owned,
        );
    Some(GatewayRouteView {
        id,
        model_pattern,
        provider,
        upstream_model,
        extra_headers,
    })
}

pub(super) fn route_to_yaml(route: &GatewayRouteView) -> Value {
    let mut map = Mapping::new();
    let id = if route.id.trim().is_empty() {
        synthesize_route_id(&route.model_pattern, &route.provider)
    } else {
        route.id.clone()
    };
    map.insert(Value::from("id"), Value::from(id));
    map.insert(
        Value::from("model_pattern"),
        Value::from(route.model_pattern.clone()),
    );
    map.insert(Value::from("provider"), Value::from(route.provider.clone()));
    if let Some(upstream) = &route.upstream_model {
        map.insert(Value::from("upstream_model"), Value::from(upstream.clone()));
    }
    if !route.extra_headers.is_empty() {
        let mut hdr = Mapping::new();
        for (k, v) in &route.extra_headers {
            hdr.insert(Value::from(k.clone()), Value::from(v.clone()));
        }
        map.insert(Value::from("extra_headers"), Value::Mapping(hdr));
    }
    Value::Mapping(map)
}

pub(super) fn ensure_gateway_mut(doc: &mut Value) -> Result<&mut Mapping, MarketplaceError> {
    let root = doc
        .as_mapping_mut()
        .ok_or_else(|| MarketplaceError::Internal("profile YAML root is not a mapping".into()))?;
    if !root.contains_key(Value::from("gateway")) {
        root.insert(Value::from("gateway"), Value::Mapping(Mapping::new()));
    }
    root.get_mut(Value::from("gateway"))
        .and_then(Value::as_mapping_mut)
        .ok_or_else(|| MarketplaceError::Internal("gateway block is not a mapping".into()))
}

pub(super) fn routes_seq_mut(doc: &mut Value) -> Result<&mut Vec<Value>, MarketplaceError> {
    let gw = ensure_gateway_mut(doc)?;
    if !gw.contains_key(Value::from("routes")) {
        gw.insert(Value::from("routes"), Value::Sequence(Vec::new()));
    }
    gw.get_mut(Value::from("routes"))
        .and_then(Value::as_sequence_mut)
        .ok_or_else(|| MarketplaceError::Internal("gateway.routes is not a sequence".into()))
}
