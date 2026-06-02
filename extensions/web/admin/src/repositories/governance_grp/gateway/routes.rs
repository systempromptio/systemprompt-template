//! Route CRUD against the profile YAML's `gateway.routes` sequence.

use std::collections::BTreeMap;
use std::path::Path;

use serde_yaml::Value;
use systemprompt_web_shared::error::MarketplaceError;

use crate::types::GatewayRouteView;

use super::matching::synthesize_route_id;
use super::yaml_io::{read_profile, route_to_yaml, routes_seq_mut, write_profile};

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
