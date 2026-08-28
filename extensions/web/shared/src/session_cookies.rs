//! The session cookie pair, and the one correct way to clear it.
//!
//! Why this is shared rather than written where it is needed: a cookie is only
//! cleared when the clearing `Set-Cookie` matches the one that set it on name,
//! `Path`, and the `Secure` attribute. Two independent implementations drifted
//! apart on exactly that — the sign-out path set `Secure` from config while the
//! bridge "use a different account" path never set it at all. A clear that
//! silently fails to evict is worst precisely there: it leaves the previous
//! session live on a screen whose next step mints a durable credential.

use axum::http::HeaderMap;
use axum::http::header::SET_COOKIE;

// Why: cookie holding the session JWT. Scoped to the whole site.
const ACCESS_COOKIE: &str = "access_token";
const ACCESS_PATH: &str = "/";

// Why: cookie holding the refresh token. Deliberately narrower in scope.
const REFRESH_COOKIE: &str = "refresh_token";
const REFRESH_PATH: &str = "/api/public/auth";

// Why: whether cookies should carry `Secure`, from the same config the setter
// reads.
fn is_secure_context() -> bool {
    systemprompt::models::Config::get().map_or(true, |c| c.use_https)
}

// Why: builds the `Set-Cookie` headers that expire both session cookies.
//
// Mirrors the setter's name, `Path`, `HttpOnly`, `SameSite` and `Secure`, so
// the browser treats these as the same cookies and actually drops them.
pub fn clear() -> HeaderMap {
    let secure_flag = if is_secure_context() { "; Secure" } else { "" };

    let mut headers = HeaderMap::new();
    for (name, path) in [(ACCESS_COOKIE, ACCESS_PATH), (REFRESH_COOKIE, REFRESH_PATH)] {
        let cookie =
            format!("{name}=; Path={path}; HttpOnly; SameSite=Lax; Max-Age=0{secure_flag}");
        if let Ok(value) = cookie.parse() {
            headers.append(SET_COOKIE, value);
        }
    }
    headers
}
