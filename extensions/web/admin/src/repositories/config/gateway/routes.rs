//! Route CRUD against the services tree's `gateway.routes` sequence.

use std::collections::BTreeMap;
use std::path::Path;

use serde_yaml::Value;
use systemprompt_web_shared::error::MarketplaceError;

use crate::types::GatewayRouteView;

use super::matching::synthesize_route_id;
use super::yaml_io::{
    read_gateway_file, route_from_yaml, route_to_yaml, routes_seq_mut, write_gateway_file,
};

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

pub fn create_route(
    gateway_path: &Path,
    route: &GatewayRouteView,
) -> Result<usize, MarketplaceError> {
    validate_route(route)?;
    let mut to_insert = route.clone();
    if to_insert.id.trim().is_empty() {
        to_insert.id = synthesize_route_id(&to_insert.model_pattern, &to_insert.provider);
    }
    let mut doc = read_gateway_file(gateway_path)?;
    let new_index = {
        let routes = routes_seq_mut(&mut doc)?;
        for existing in routes.iter() {
            // Why: the effective id, not the literal `id:` key — a route that
            // omits it still owns its synthesized id, and a create that reused
            // it would mint two routes the ACL cannot tell apart.
            if route_from_yaml(existing).is_some_and(|r| r.id == to_insert.id) {
                return Err(MarketplaceError::BadRequest(format!(
                    "route id `{}` already exists",
                    to_insert.id
                )));
            }
        }
        routes.push(route_to_yaml(&to_insert));
        routes.len() - 1
    };
    write_gateway_file(gateway_path, &doc)?;
    Ok(new_index)
}

pub fn update_route(
    gateway_path: &Path,
    index: usize,
    route: &GatewayRouteView,
) -> Result<bool, MarketplaceError> {
    validate_route(route)?;
    let mut doc = read_gateway_file(gateway_path)?;
    {
        let routes = routes_seq_mut(&mut doc)?;
        if index >= routes.len() {
            return Ok(false);
        }
        let mut merged = route.clone();
        let had_explicit_id = routes[index]
            .as_mapping()
            .is_some_and(|m| m.contains_key(Value::from("id")));
        if let Some(existing) = routes[index].as_mapping() {
            // Why: an update body that omits pricing/when/requires must keep
            // the on-disk blocks — the admin UI never round-trips them.
            merged.pricing = merged
                .pricing
                .or_else(|| existing.get(Value::from("pricing")).cloned());
            merged.when = merged
                .when
                .or_else(|| existing.get(Value::from("when")).cloned());
            merged.requires = merged
                .requires
                .or_else(|| existing.get(Value::from("requires")).cloned());
        }
        routes[index] = route_to_yaml(&merged);
        // Why: an edit adds no field the operator did not write. A route that
        // carried no `id:` keeps carrying none even when the edit changes the
        // id it synthesizes to, which is what changing `provider` does.
        if !had_explicit_id && let Some(map) = routes[index].as_mapping_mut() {
            map.shift_remove(Value::from("id"));
        }
    }
    write_gateway_file(gateway_path, &doc)?;
    Ok(true)
}

pub fn delete_route(gateway_path: &Path, index: usize) -> Result<bool, MarketplaceError> {
    let mut doc = read_gateway_file(gateway_path)?;
    {
        let routes = routes_seq_mut(&mut doc)?;
        if index >= routes.len() {
            return Ok(false);
        }
        routes.remove(index);
    }
    write_gateway_file(gateway_path, &doc)?;
    Ok(true)
}

pub fn reorder_routes(gateway_path: &Path, order: &[usize]) -> Result<(), MarketplaceError> {
    let mut doc = read_gateway_file(gateway_path)?;
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
    write_gateway_file(gateway_path, &doc)?;
    Ok(())
}

// Why: explicit maintenance persists IDs; ordinary reads and edits derive them
// in memory.
pub fn ensure_route_ids(config_path: &Path) -> Result<bool, MarketplaceError> {
    let mut doc = read_gateway_file(config_path)?;
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
            .to_owned();
        let provider = map
            .get(Value::from("provider"))
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned();
        let id = synthesize_route_id(&model_pattern, &provider);
        map.insert(Value::from("id"), Value::from(id));
        changed = true;
    }
    if changed {
        write_gateway_file(config_path, &doc)?;
    }
    Ok(changed)
}
