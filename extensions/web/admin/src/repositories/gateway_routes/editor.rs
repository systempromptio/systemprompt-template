//! The `/admin/gateway` editor's writes: create, update, delete and reorder,
//! addressed by position as the page addresses them.
//!
//! Every write lands in `gateway_routes` stamped `dashboard`, renumbers the
//! positions so they stay `0..n`, and regenerates the file core boots from.
//! A route the declaration did not give an `id:` is keyed by the id the
//! loader synthesises from its pattern and provider; editing either re-keys
//! the row the way the loader would, so the file and the table agree on
//! what the route is called and the access rules that name it still match.

use std::path::Path;

use sqlx::PgPool;

use super::render::regenerate_gateway_file;
use super::rows::{
    RouteRow, RouteWrite, SOURCE_DASHBOARD, delete_gateway_route, list_gateway_routes,
    set_gateway_route_position, upsert_gateway_route,
};
use crate::error::{AdminError, AdminResult};
use crate::repositories::config::gateway::{
    normalise_metadata, synthesize_route_id, validate_route,
};
use crate::types::GatewayRouteView;

async fn renumber(pool: &PgPool, rows: &[RouteRow]) -> AdminResult<()> {
    for (i, r) in rows.iter().enumerate() {
        let position = i32::try_from(i).unwrap_or(i32::MAX);
        if r.position != position {
            set_gateway_route_position(pool, &r.route.id, position).await?;
        }
    }
    Ok(())
}

fn at(rows: &[RouteRow], index: usize) -> AdminResult<&RouteRow> {
    rows.get(index)
        .ok_or_else(|| AdminError::NotFound("Route not found".to_owned()))
}

pub async fn create_route_entry(
    pool: &PgPool,
    gateway_path: &Path,
    route: &GatewayRouteView,
) -> AdminResult<usize> {
    validate_route(route)?;
    let mut to_insert = route.clone();
    normalise_metadata(&mut to_insert);
    let explicit_id = !to_insert.id.trim().is_empty();
    if !explicit_id {
        to_insert.id = synthesize_route_id(&to_insert.model_pattern, &to_insert.provider);
    }
    let rows = list_gateway_routes(pool).await?;
    if rows.iter().any(|r| r.route.id == to_insert.id) {
        return Err(AdminError::BadRequest(format!(
            "route id `{}` already exists",
            to_insert.id
        )));
    }
    let position = i32::try_from(rows.len()).unwrap_or(i32::MAX);
    upsert_gateway_route(
        pool,
        &RouteWrite {
            route: &to_insert,
            position,
            explicit_id,
            source: SOURCE_DASHBOARD,
        },
    )
    .await?;
    regenerate_gateway_file(pool, gateway_path).await?;
    Ok(rows.len())
}

pub async fn update_route_at(
    pool: &PgPool,
    gateway_path: &Path,
    index: usize,
    route: &GatewayRouteView,
) -> AdminResult<()> {
    validate_route(route)?;
    let rows = list_gateway_routes(pool).await?;
    let existing = at(&rows, index)?;
    let mut merged = route.clone();
    normalise_metadata(&mut merged);
    // Why: an update body that omits the blocks the form does not render
    // must keep them — dropping pricing/when/requires on a save would
    // silently rewrite routing policy.
    let keep = &existing.route;
    merged.pricing = merged.pricing.or_else(|| keep.pricing.clone());
    merged.when = merged.when.or_else(|| keep.when.clone());
    merged.requires = merged.requires.or_else(|| keep.requires.clone());
    // Why: the form renders no failover fields either; a save that omits
    // them must not silently strip a route's fallback.
    merged.fallback_provider = merged
        .fallback_provider
        .or_else(|| keep.fallback_provider.clone());
    merged.fallback_upstream_model = merged
        .fallback_upstream_model
        .or_else(|| keep.fallback_upstream_model.clone());
    if merged.extra_headers.is_empty() {
        merged.extra_headers = keep.extra_headers.clone();
    }
    let derived = synthesize_route_id(&merged.model_pattern, &merged.provider);
    let body_id = merged.id.trim().to_owned();
    let explicit_id =
        existing.explicit_id || (!body_id.is_empty() && body_id != keep.id && body_id != derived);
    merged.id = if explicit_id && !body_id.is_empty() {
        body_id
    } else if existing.explicit_id {
        keep.id.clone()
    } else {
        derived
    };
    if merged.id != keep.id {
        if rows.iter().any(|r| r.route.id == merged.id) {
            return Err(AdminError::BadRequest(format!(
                "route id `{}` already exists",
                merged.id
            )));
        }
        delete_gateway_route(pool, &keep.id).await?;
    }
    upsert_gateway_route(
        pool,
        &RouteWrite {
            route: &merged,
            position: existing.position,
            explicit_id,
            source: SOURCE_DASHBOARD,
        },
    )
    .await?;
    regenerate_gateway_file(pool, gateway_path).await?;
    Ok(())
}

pub async fn delete_route_at(pool: &PgPool, gateway_path: &Path, index: usize) -> AdminResult<()> {
    let rows = list_gateway_routes(pool).await?;
    let target = at(&rows, index)?;
    delete_gateway_route(pool, &target.route.id).await?;
    renumber(pool, &list_gateway_routes(pool).await?).await?;
    regenerate_gateway_file(pool, gateway_path).await?;
    Ok(())
}

pub async fn reorder_route_positions(
    pool: &PgPool,
    gateway_path: &Path,
    order: &[usize],
) -> AdminResult<()> {
    let rows = list_gateway_routes(pool).await?;
    let n = rows.len();
    if order.len() != n {
        return Err(AdminError::BadRequest(format!(
            "order has {} entries but there are {n} routes",
            order.len()
        )));
    }
    let mut seen = vec![false; n];
    for &i in order {
        if i >= n || seen[i] {
            return Err(AdminError::BadRequest(
                "order must be a permutation of route indices".to_owned(),
            ));
        }
        seen[i] = true;
    }
    for (position, &i) in order.iter().enumerate() {
        let position = i32::try_from(position).unwrap_or(i32::MAX);
        set_gateway_route_position(pool, &rows[i].route.id, position).await?;
    }
    regenerate_gateway_file(pool, gateway_path).await?;
    Ok(())
}
