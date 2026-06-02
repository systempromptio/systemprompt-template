use std::collections::hash_map::DefaultHasher;
use std::collections::BTreeMap;
use std::hash::{Hash, Hasher};
use std::path::Path;

use serde_yaml::{Mapping, Value};
use systemprompt_web_shared::error::MarketplaceError;

use crate::types::{GatewayConfigView, GatewayRouteView, UpdateGatewaySettingsRequest};

const DEFAULT_AUTH_SCHEME: &str = "bearer";
const DEFAULT_INFERENCE_PATH_PREFIX: &str = "/v1";

fn read_profile(profile_path: &Path) -> Result<Value, MarketplaceError> {
    let content = std::fs::read_to_string(profile_path)?;
    let doc: Value = serde_yaml::from_str(&content)?;
    Ok(doc)
}

fn write_profile(profile_path: &Path, doc: &Value) -> Result<(), MarketplaceError> {
    let yaml_str = serde_yaml::to_string(doc)?;
    std::fs::write(profile_path, yaml_str).map_err(|e| {
        MarketplaceError::Internal(format!(
            "Failed to write profile {}: {e}",
            profile_path.display()
        ))
    })?;
    Ok(())
}

fn route_from_yaml(val: &Value) -> Option<GatewayRouteView> {
    let map = val.as_mapping()?;
    let model_pattern = map.get(Value::from("model_pattern"))?.as_str()?.to_string();
    let provider = map.get(Value::from("provider"))?.as_str()?.to_string();
    let upstream_model = map
        .get(Value::from("upstream_model"))
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let extra_headers = map
        .get(Value::from("extra_headers"))
        .and_then(Value::as_mapping)
        .map(|m| {
            m.iter()
                .filter_map(|(k, v)| Some((k.as_str()?.to_string(), v.as_str()?.to_string())))
                .collect()
        })
        .unwrap_or_default();
    let id = map
        .get(Value::from("id"))
        .and_then(Value::as_str)
        .map_or_else(
            || synthesize_route_id(&model_pattern, &provider),
            ToString::to_string,
        );
    Some(GatewayRouteView {
        id,
        model_pattern,
        provider,
        upstream_model,
        extra_headers,
    })
}

pub fn slugify_pattern(pattern: &str) -> String {
    let mut out = String::with_capacity(pattern.len());
    let mut last_dash = false;
    for ch in pattern.chars() {
        let mapped: Option<&str> = if ch == '*' {
            Some("star")
        } else if ch.is_ascii_alphanumeric() {
            None
        } else {
            Some("-")
        };
        match mapped {
            Some("-") => {
                if !last_dash && !out.is_empty() {
                    out.push('-');
                    last_dash = true;
                }
            }
            Some(s) => {
                out.push_str(s);
                last_dash = false;
            }
            None => {
                for lc in ch.to_lowercase() {
                    out.push(lc);
                }
                last_dash = false;
            }
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    while out.starts_with('-') {
        out.remove(0);
    }
    if out.is_empty() {
        out.push_str("route");
    }
    out
}

pub fn synthesize_route_id(model_pattern: &str, provider: &str) -> String {
    let mut hasher = DefaultHasher::new();
    model_pattern.hash(&mut hasher);
    provider.hash(&mut hasher);
    let h = hasher.finish();
    let hash6: String = format!("{h:016x}").chars().take(6).collect();
    format!("{}-{}", slugify_pattern(model_pattern), hash6)
}

fn route_to_yaml(route: &GatewayRouteView) -> Value {
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

pub fn validate_route(route: &GatewayRouteView) -> Result<(), MarketplaceError> {
    if route.model_pattern.trim().is_empty() {
        return Err(MarketplaceError::BadRequest(
            "model_pattern is required".into(),
        ));
    }
    if route.provider.trim().is_empty() {
        return Err(MarketplaceError::BadRequest("provider is required".into()));
    }
    Ok(())
}

/// Ensure every route in the profile has an explicit stable `id`, persisting
/// synthesized ids back to disk if any were missing. Returns true when the
/// profile was rewritten.
pub fn ensure_route_ids(profile_path: &Path) -> Result<bool, MarketplaceError> {
    let mut doc = read_profile(profile_path)?;
    let mut changed = false;
    let Some(gateway) = doc
        .as_mapping_mut()
        .and_then(|m| m.get_mut(Value::from("gateway")))
    else {
        return Ok(false);
    };
    let Some(routes) = gateway
        .as_mapping_mut()
        .and_then(|g| g.get_mut(Value::from("routes")))
        .and_then(Value::as_sequence_mut)
    else {
        return Ok(false);
    };
    for route in routes.iter_mut() {
        let Some(map) = route.as_mapping_mut() else {
            continue;
        };
        let has_id = map
            .get(Value::from("id"))
            .and_then(Value::as_str)
            .is_some_and(|s| !s.trim().is_empty());
        if has_id {
            continue;
        }
        let model_pattern = map
            .get(Value::from("model_pattern"))
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        let provider = map
            .get(Value::from("provider"))
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        let id = synthesize_route_id(&model_pattern, &provider);
        map.insert(Value::from("id"), Value::from(id));
        changed = true;
    }
    if changed {
        write_profile(profile_path, &doc)?;
    }
    Ok(changed)
}

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
        .to_string();
    let inference_path_prefix = gateway
        .and_then(|g| g.get("inference_path_prefix"))
        .and_then(Value::as_str)
        .unwrap_or(DEFAULT_INFERENCE_PATH_PREFIX)
        .to_string();
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

fn ensure_gateway_mut(doc: &mut Value) -> Result<&mut Mapping, MarketplaceError> {
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

fn routes_seq_mut(doc: &mut Value) -> Result<&mut Vec<Value>, MarketplaceError> {
    let gw = ensure_gateway_mut(doc)?;
    if !gw.contains_key(Value::from("routes")) {
        gw.insert(Value::from("routes"), Value::Sequence(Vec::new()));
    }
    gw.get_mut(Value::from("routes"))
        .and_then(Value::as_sequence_mut)
        .ok_or_else(|| MarketplaceError::Internal("gateway.routes is not a sequence".into()))
}

pub fn create_route(
    profile_path: &Path,
    route: &GatewayRouteView,
) -> Result<usize, MarketplaceError> {
    validate_route(route)?;
    ensure_route_ids(profile_path)?;
    let mut to_insert = route.clone();
    if to_insert.id.trim().is_empty() {
        to_insert.id = synthesize_route_id(&to_insert.model_pattern, &to_insert.provider);
    }
    let mut doc = read_profile(profile_path)?;
    let new_index = {
        let routes = routes_seq_mut(&mut doc)?;
        for existing in routes.iter() {
            if existing
                .as_mapping()
                .and_then(|m| m.get(Value::from("id")))
                .and_then(Value::as_str)
                == Some(to_insert.id.as_str())
            {
                return Err(MarketplaceError::BadRequest(format!(
                    "route id `{}` already exists",
                    to_insert.id
                )));
            }
        }
        routes.push(route_to_yaml(&to_insert));
        routes.len() - 1
    };
    write_profile(profile_path, &doc)?;
    Ok(new_index)
}

pub fn update_route(
    profile_path: &Path,
    index: usize,
    route: &GatewayRouteView,
) -> Result<bool, MarketplaceError> {
    validate_route(route)?;
    let mut doc = read_profile(profile_path)?;
    {
        let routes = routes_seq_mut(&mut doc)?;
        if index >= routes.len() {
            return Ok(false);
        }
        routes[index] = route_to_yaml(route);
    }
    write_profile(profile_path, &doc)?;
    Ok(true)
}

pub fn delete_route(profile_path: &Path, index: usize) -> Result<bool, MarketplaceError> {
    let mut doc = read_profile(profile_path)?;
    {
        let routes = routes_seq_mut(&mut doc)?;
        if index >= routes.len() {
            return Ok(false);
        }
        routes.remove(index);
    }
    write_profile(profile_path, &doc)?;
    Ok(true)
}

pub fn reorder_routes(profile_path: &Path, order: &[usize]) -> Result<(), MarketplaceError> {
    let mut doc = read_profile(profile_path)?;
    {
        let routes = routes_seq_mut(&mut doc)?;
        let n = routes.len();
        if order.len() != n {
            return Err(MarketplaceError::BadRequest(format!(
                "order has {} entries but there are {n} routes",
                order.len()
            )));
        }
        let mut seen = vec![false; n];
        for &i in order {
            if i >= n || seen[i] {
                return Err(MarketplaceError::BadRequest(
                    "order must be a permutation of route indices".into(),
                ));
            }
            seen[i] = true;
        }
        let mut by_index: BTreeMap<usize, Value> =
            std::mem::take(routes).into_iter().enumerate().collect();
        for &i in order {
            if let Some(v) = by_index.remove(&i) {
                routes.push(v);
            }
        }
    }
    write_profile(profile_path, &doc)?;
    Ok(())
}

/// Best-effort: which route index (if any) would match the given model string,
/// using the same first-match-wins glob semantics the gateway uses.
#[must_use]
pub fn find_matching_route_index(routes: &[GatewayRouteView], model: &str) -> Option<usize> {
    routes
        .iter()
        .position(|r| glob_match(&r.model_pattern, model))
}

/// Sibling to [`find_matching_route_index`] that returns the route reference
/// directly. Useful for ACL lookups where the caller wants the stable `id`.
#[must_use]
pub fn find_matching_route<'a>(
    routes: &'a [GatewayRouteView],
    model: &str,
) -> Option<&'a GatewayRouteView> {
    routes.iter().find(|r| glob_match(&r.model_pattern, model))
}

#[must_use]
pub fn find_route_index_by_id(routes: &[GatewayRouteView], id: &str) -> Option<usize> {
    routes.iter().position(|r| r.id == id)
}

pub fn glob_match(pattern: &str, value: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    if !pattern.contains('*') {
        return pattern == value;
    }
    let parts: Vec<&str> = pattern.split('*').collect();
    if parts.len() == 2 {
        let (prefix, suffix) = (parts[0], parts[1]);
        return value.starts_with(prefix)
            && value.ends_with(suffix)
            && value.len() >= prefix.len() + suffix.len();
    }
    let mut cursor = 0usize;
    for (i, segment) in parts.iter().enumerate() {
        if segment.is_empty() {
            continue;
        }
        let Some(found) = value[cursor..].find(segment) else {
            return false;
        };
        if i == 0 && found != 0 {
            return false;
        }
        cursor += found + segment.len();
    }
    if let Some(last) = parts.last() {
        if !last.is_empty() && !value.ends_with(last) {
            return false;
        }
    }
    true
}
