//! Exhaustiveness: the routes the router actually mounts, recovered from the
//! route modules themselves.
//!
//! Axum exposes no way to enumerate a built `Router`, so the method/path pairs
//! are read back from the source that declares them — pulled in with
//! `include_str!`, so the suite and the router can never disagree about what
//! exists. Deriving the table rather than hand-writing it is what makes
//! coverage exhaustive by construction: a new `.route(...)` is picked up and
//! exercised on the next run, and shows up as a baseline addition.

use crate::app::{ADMIN_API_PREFIX, BRIDGE_PREFIX, SSR_PREFIX};

const ADMIN_API_SRC: &str = include_str!("../../../../extensions/web/admin/src/routes/admin.rs");
const SSR_SRC: &str = include_str!("../../../../extensions/web/admin/src/routes/ssr.rs");
const BRIDGE_SRC: &str = include_str!("../../../../extensions/web/admin/src/routes/ssr_bridge.rs");

const METHODS: [&str; 5] = ["get", "post", "put", "patch", "delete"];

// A single method/path pair the router serves.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct MountedRoute {
    pub method: String,
    // The path as mounted, path parameters still in `{brace}` form.
    pub template: String,
}

impl MountedRoute {
    // A concrete URL for this route.
    //
    // Every path parameter is filled with a well-formed id that exists in no
    // table. That is the interesting case rather than a limitation: a route
    // asked for an id it cannot find owes the caller a 404, and one that
    // answers 500 instead is the exact defect this suite is here to catch.
    pub fn request_path(&self) -> String {
        self.template
            .split('/')
            .map(|seg| {
                if seg.starts_with('{') && seg.ends_with('}') {
                    UNKNOWN_ID
                } else {
                    seg
                }
            })
            .collect::<Vec<_>>()
            .join("/")
    }

    pub fn key(&self) -> String {
        format!("{} {}", self.method.to_uppercase(), self.template)
    }
}

// Well-formed, and deliberately absent from every table.
const UNKNOWN_ID: &str = "00000000-0000-4000-8000-000000000000";

pub fn mounted_routes() -> Vec<MountedRoute> {
    let mut routes = Vec::new();
    for (src, prefix) in [
        (ADMIN_API_SRC, ADMIN_API_PREFIX),
        (SSR_SRC, SSR_PREFIX),
        (BRIDGE_SRC, BRIDGE_PREFIX),
    ] {
        parse(src, prefix, &mut routes);
    }
    routes.sort();
    routes.dedup();
    routes
}

// Split the source on `.route(` and read each call's path literal plus every
// method constructor applied to it.
fn parse(src: &str, prefix: &str, out: &mut Vec<MountedRoute>) {
    let calls: Vec<usize> = src.match_indices(".route(").map(|(i, _)| i).collect();
    for (n, &start) in calls.iter().enumerate() {
        let end = calls.get(n + 1).copied().unwrap_or(src.len());
        let segment = &src[start..end];

        let Some(template) = path_literal(segment) else {
            continue;
        };
        let mounted = if template == "/" {
            prefix.to_owned()
        } else {
            format!("{prefix}{template}")
        };

        for method in methods_in(segment) {
            out.push(MountedRoute {
                method,
                template: mounted.clone(),
            });
        }
    }
}

fn path_literal(segment: &str) -> Option<String> {
    let open = segment.find('"')?;
    let after = &segment[open + 1..];
    let close = after.find('"')?;
    let literal = &after[..close];
    assert!(
        literal.starts_with('/'),
        "route literal {literal:?} is not a path — the route-source parser needs updating"
    );
    Some(literal.to_owned())
}

// Method constructors applied within one `.route(...)` call, in declaration
// order. Matches only a bare `name(`, so handler identifiers that merely
// contain a method name (`get_gateway_handler`) are not mistaken for one.
fn methods_in(segment: &str) -> Vec<String> {
    let bytes = segment.as_bytes();
    let mut found = Vec::new();
    for method in METHODS {
        let needle = format!("{method}(");
        for (idx, _) in segment.match_indices(&needle) {
            let preceded_by_ident = idx > 0 && {
                let prev = bytes[idx - 1];
                prev.is_ascii_alphanumeric() || prev == b'_'
            };
            if !preceded_by_ident && !found.contains(&method.to_owned()) {
                found.push(method.to_owned());
            }
        }
    }
    assert!(
        !found.is_empty(),
        "no HTTP method found in route call: {segment:?}"
    );
    found
}
