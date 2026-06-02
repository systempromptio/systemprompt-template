//! Route-id synthesis and glob matching.
//!
//! Route ids are stable, slug-based identifiers derived from the model pattern
//! plus a short hash of `(model_pattern, provider)`. [`glob_match`] implements
//! the same first-match-wins `*` semantics the gateway uses at request time.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::types::GatewayRouteView;

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
