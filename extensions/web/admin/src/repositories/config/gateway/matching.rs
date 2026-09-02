//! Route-id synthesis and glob matching.
//!
//! Route ids are stable, slug-based identifiers derived from the model pattern
//! plus a short hash of `(model_pattern, provider)`. [`glob_match`] implements
//! the same first-match-wins `*` semantics the gateway uses at request time.
//!
//! Synthesis itself belongs to core and lives in
//! `systemprompt::models::services`. A second implementation here would only
//! ever agree with core's by accident — the id is a hash, and any drift in the
//! algorithm mints ids the gateway cannot dispatch — so the functions below
//! delegate. They exist only to keep the `String`-returning signature this
//! crate's callers expect, rather than core's `RouteId`.

use crate::types::GatewayRouteView;

#[must_use]
pub fn slugify_pattern(pattern: &str) -> String {
    systemprompt::models::services::slugify_pattern(pattern)
}

// Why: `String` rather than core's `RouteId` because every caller here writes
// the id straight into services YAML as a scalar.
#[must_use]
pub fn synthesize_route_id(model_pattern: &str, provider: &str) -> String {
    systemprompt::models::services::synthesize_route_id(model_pattern, provider)
        .as_str()
        .to_owned()
}

// Why: Best-effort: which route index (if any) would match the given model
// string, using the same first-match-wins glob semantics the gateway uses.
#[must_use]
pub fn find_matching_route_index(routes: &[GatewayRouteView], model: &str) -> Option<usize> {
    routes
        .iter()
        .position(|r| glob_match(&r.model_pattern, model))
}

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
    if let Some(last) = parts.last()
        && !last.is_empty()
        && !value.ends_with(last)
    {
        return false;
    }
    true
}
