//! The ADFS flow-state cookie: the attributes that decide whether it survives
//! the cross-site assertion POST, and the set/clear agreement that decides
//! whether it is ever removed.

use axum::http::HeaderMap;
use axum::http::header::COOKIE;
use systemprompt_web_admin::{FlowState, clear_state_cookie, read_state_cookie, state_cookie};

fn flow() -> FlowState {
    FlowState {
        state: "nonce-abc".to_owned(),
        request_id: "_1234abcd".to_owned(),
        issued_at_unix: 1_756_000_000,
        redirect_to: "/admin/analytics".to_owned(),
    }
}

// Why: the regression. The assertion consumer service is reached by a
// cross-site form POST from the AD FS farm; `SameSite=Lax` withholds the
// cookie on one, the SP then holds no tracker, and the `saml` crate rejects
// the solicited response as unsolicited.
#[test]
fn the_state_cookie_survives_a_cross_site_assertion_post() {
    let cookie = state_cookie(&flow(), true);
    assert!(cookie.contains("SameSite=None"), "got: {cookie}");
    assert!(cookie.contains("Secure"), "got: {cookie}");
    assert!(!cookie.contains("SameSite=Lax"), "got: {cookie}");
}

// Why: `SameSite=None` without `Secure` is dropped outright, so plain http
// keeps `Lax` rather than shipping a cookie no browser will store.
#[test]
fn plain_http_falls_back_to_lax_rather_than_an_unstorable_cookie() {
    let cookie = state_cookie(&flow(), false);
    assert!(cookie.contains("SameSite=Lax"), "got: {cookie}");
    assert!(!cookie.contains("Secure"), "got: {cookie}");
}

// Why: a clear only takes effect when its attributes match the cookie that was
// set; a drift between the two leaves the spent state cookie in the browser.
#[test]
fn clearing_matches_the_attributes_the_cookie_was_set_with() {
    for secure in [true, false] {
        let set = state_cookie(&flow(), secure);
        let clear = clear_state_cookie(secure);
        for attr in ["Path=/admin/auth/adfs", "HttpOnly"] {
            assert!(
                set.contains(attr) && clear.contains(attr),
                "{attr} / {secure}"
            );
        }
        let same_site = if secure {
            "SameSite=None"
        } else {
            "SameSite=Lax"
        };
        assert!(clear.contains(same_site), "got: {clear}");
        assert_eq!(set.contains("Secure"), clear.contains("Secure"));
        assert!(clear.contains("Max-Age=0"), "got: {clear}");
    }
}

// Why: the cookie is parsed back by the callback, so the four '|'-separated
// segments must round-trip exactly as written.
#[test]
fn the_flow_state_round_trips_through_the_cookie() {
    let expected = flow();
    let value = state_cookie(&expected, true);
    let pair = value.split(';').next().expect("cookie pair");

    let mut headers = HeaderMap::new();
    headers.insert(COOKIE, pair.parse().expect("header value"));

    assert_eq!(read_state_cookie(&headers), Some(expected));
}

// Why: the callback reads one cookie out of whatever else the browser sends.
#[test]
fn the_flow_state_is_found_among_other_cookies() {
    let expected = flow();
    let pair = state_cookie(&expected, true)
        .split(';')
        .next()
        .expect("cookie pair")
        .to_owned();

    let mut headers = HeaderMap::new();
    headers.insert(
        COOKIE,
        format!("other=1; {pair}; access_token=xyz")
            .parse()
            .expect("header value"),
    );

    assert_eq!(read_state_cookie(&headers), Some(expected));
}
