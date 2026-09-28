//! Route CRUD against the services YAML's `gateway.routes` sequence, and the
//! whole-sequence replace the `gateway_routes` table uses to regenerate it.

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

// Why: the form posts every field, so a cleared name arrives as `""`; the
// file and the mirror should carry an absent name, not an empty one.
pub fn normalise_metadata(route: &mut GatewayRouteView) {
    let clean = |v: &mut Option<String>| {
        *v = v
            .as_deref()
            .map(str::trim)
            .filter(|t| !t.is_empty())
            .map(str::to_owned);
    };
    clean(&mut route.name);
    clean(&mut route.description);
}

pub fn create_route(
    gateway_path: &Path,
    route: &GatewayRouteView,
) -> Result<usize, MarketplaceError> {
    validate_route(route)?;
    let mut to_insert = route.clone();
    normalise_metadata(&mut to_insert);
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
        normalise_metadata(&mut merged);
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
            let text = |key: &str| {
                existing
                    .get(Value::from(key))
                    .and_then(Value::as_str)
                    .map(str::to_owned)
            };
            merged.fallback_provider = merged
                .fallback_provider
                .or_else(|| text("fallback_provider"));
            merged.fallback_upstream_model = merged
                .fallback_upstream_model
                .or_else(|| text("fallback_upstream_model"));
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

// Why: the database → file direction. Every other function here edits one
// route of the file in place; this one makes the file's `routes:` sequence
// equal the given list, which is what the `gateway_routes` table does after
// each console write so the next restart dispatches what the table says.
// The settings above the sequence and the header comment are untouched.
pub fn replace_routes(
    gateway_path: &Path,
    routes: &[GatewayRouteView],
) -> Result<(), MarketplaceError> {
    for route in routes {
        validate_route(route)?;
    }
    let mut doc = read_gateway_file(gateway_path)?;
    {
        let seq = routes_seq_mut(&mut doc)?;
        seq.clear();
        seq.extend(routes.iter().map(route_to_yaml));
    }
    write_gateway_file(gateway_path, &doc)
}
