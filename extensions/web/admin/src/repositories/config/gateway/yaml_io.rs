//! Gateway YAML read/write and route <-> YAML conversion.
//!
//! All mutation paths funnel through [`read_gateway_file`] /
//! [`write_gateway_file`] so the `gateway` block stays well-formed, and through
//! [`ensure_gateway_mut`] / [`routes_seq_mut`] which lazily create the block
//! and `routes` sequence.
//!
//! `serde_yaml` round-trips values, not text, so a naive write drops every
//! comment in the file. [`write_gateway_file`] carries the operator's header
//! comment across, and [`route_to_yaml`] omits an `id` it could synthesize, so
//! an admin edit changes the field it edited and nothing else.

use std::path::Path;

use serde_yaml::{Mapping, Value};
use systemprompt_web_shared::error::MarketplaceError;

use crate::types::GatewayRouteView;

use super::matching::synthesize_route_id;

pub(super) fn read_gateway_file(gateway_path: &Path) -> Result<Value, MarketplaceError> {
    let content = std::fs::read_to_string(gateway_path)?;
    let doc: Value = serde_yaml::from_str(&content)?;
    Ok(doc)
}

pub(super) fn write_gateway_file(gateway_path: &Path, doc: &Value) -> Result<(), MarketplaceError> {
    let header = leading_comment_header(gateway_path);
    let yaml_str = format!("{header}{}", serde_yaml::to_string(doc)?);
    std::fs::write(gateway_path, yaml_str)
        .map_err(|e| MarketplaceError::config_file(gateway_path.display().to_string(), e))?;
    Ok(())
}

// Why: the header names the file's purpose and the CLI that edits it. It is
// the only comment block whose position survives a value round-trip, because
// nothing above it can move; inline comments cannot be anchored to a `Value`
// and are lost, which is why the editor writes as little as it can.
fn leading_comment_header(gateway_path: &Path) -> String {
    let Ok(content) = std::fs::read_to_string(gateway_path) else {
        return String::new();
    };
    content
        .lines()
        .take_while(|line| line.trim_start().starts_with('#'))
        .fold(String::new(), |mut acc, line| {
            acc.push_str(line);
            acc.push('\n');
            acc
        })
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
        pricing: map.get(Value::from("pricing")).cloned(),
        when: map.get(Value::from("when")).cloned(),
        requires: map.get(Value::from("requires")).cloned(),
    })
}

// Why: `id` is written only when it is not derivable. Synthesis is
// deterministic in `(model_pattern, provider)`, so a synthesized id in the file
// is noise the operator did not write and `route_from_yaml` recreates it on
// every read. A hand-chosen id is data and is always kept.
pub(super) fn route_to_yaml(route: &GatewayRouteView) -> Value {
    let mut map = Mapping::new();
    let derived = synthesize_route_id(&route.model_pattern, &route.provider);
    let id = route.id.trim();
    if !id.is_empty() && id != derived {
        map.insert(Value::from("id"), Value::from(id.to_owned()));
    }
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
    if let Some(pricing) = &route.pricing {
        map.insert(Value::from("pricing"), pricing.clone());
    }
    if let Some(when) = &route.when {
        map.insert(Value::from("when"), when.clone());
    }
    if let Some(requires) = &route.requires {
        map.insert(Value::from("requires"), requires.clone());
    }
    Value::Mapping(map)
}

pub(super) fn ensure_gateway_mut(doc: &mut Value) -> Result<&mut Mapping, MarketplaceError> {
    let root = doc
        .as_mapping_mut()
        .ok_or_else(|| MarketplaceError::Internal("gateway YAML root is not a mapping".into()))?;
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
