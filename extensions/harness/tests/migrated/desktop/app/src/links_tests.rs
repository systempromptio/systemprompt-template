//! Links into the app: unsupported ones are refused, and a Settings page
//! survives its linked form.

use systemprompt_harness_views::app::deeplink::{Link, LinkError, parse};

const SCHEME: &str = "harness-test";

#[test]
fn an_unsupported_link_is_refused_and_opens_nothing() {
    assert!(matches!(
        parse(SCHEME, "harness-test://launch-rockets"),
        Err(LinkError::UnknownAction { .. })
    ));
    assert!(matches!(
        parse(SCHEME, "other://open?dir=/tmp"),
        Err(LinkError::ForeignScheme { .. })
    ));
}

#[test]
fn a_settings_page_round_trips_through_its_link() {
    for page in ["general", "permissions", "about"] {
        let link = format!("{SCHEME}://settings?page={page}");
        assert_eq!(
            parse(SCHEME, &link),
            Ok(Link::OpenSettings {
                page: Some(page.to_owned())
            })
        );
    }
    assert_eq!(
        parse(SCHEME, "harness-test://settings"),
        Ok(Link::OpenSettings { page: None })
    );
}
