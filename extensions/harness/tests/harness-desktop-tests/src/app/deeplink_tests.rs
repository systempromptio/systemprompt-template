use std::path::PathBuf;

use systemprompt_engine::store::SessionId;
use systemprompt_harness_views::app::deeplink::{
    ArgsError, LaunchRequest, Link, LinkError, is_link, parse, parse_args,
};

use crate::TestResult;

const SCHEME: &str = "sp-harness";

fn args(list: &[&str]) -> Result<LaunchRequest, ArgsError> {
    parse_args(SCHEME, list.iter().map(|value| (*value).to_owned()))
}

#[test]
fn links_need_this_apps_scheme() {
    assert!(is_link(SCHEME, "sp-harness://open?dir=x"));
    assert!(is_link(SCHEME, "SP-HARNESS://open?dir=x"));
    assert!(!is_link(SCHEME, "sp-harness-dev://open?dir=x"));
    assert!(!is_link(SCHEME, r"C:\work"));
    assert_eq!(
        parse(SCHEME, "other://open?dir=x"),
        Err(LinkError::ForeignScheme {
            found: "other".to_owned()
        })
    );
}

#[test]
fn open_links_carry_their_directory() {
    assert_eq!(
        parse(SCHEME, "sp-harness://open?dir=C%3A%5Cwork%5Capp"),
        Ok(Link::OpenProject {
            directory: PathBuf::from(r"C:\work\app")
        })
    );
    assert_eq!(
        parse(SCHEME, "sp-harness://open"),
        Err(LinkError::MissingParameter { name: "dir" })
    );
}

#[test]
fn new_session_links_take_an_optional_directory_and_prompt() {
    assert_eq!(
        parse(SCHEME, "sp-harness://new?prompt=fix%20the%20tests"),
        Ok(Link::NewSession {
            directory: None,
            prompt: Some("fix the tests".to_owned())
        })
    );
    assert_eq!(
        parse(SCHEME, "sp-harness://new?dir=%2Fwork&prompt="),
        Ok(Link::NewSession {
            directory: Some(PathBuf::from("/work")),
            prompt: None
        })
    );
}

#[test]
fn session_links_need_a_valid_id() {
    let id = SessionId::mint(&crate::IDS);
    assert_eq!(
        parse(SCHEME, &format!("sp-harness://session?id={id}")),
        Ok(Link::OpenSession { session: id })
    );
    assert!(matches!(
        parse(SCHEME, "sp-harness://session?id=nope"),
        Err(LinkError::InvalidSession(_))
    ));
}

#[test]
fn settings_links_may_name_a_page() {
    assert_eq!(
        parse(SCHEME, "sp-harness://settings"),
        Ok(Link::OpenSettings { page: None })
    );
    assert_eq!(
        parse(SCHEME, "sp-harness://settings?page=governance"),
        Ok(Link::OpenSettings {
            page: Some("governance".to_owned())
        })
    );
    assert_eq!(
        parse(SCHEME, "sp-harness://launch"),
        Err(LinkError::UnknownAction {
            action: "launch".to_owned()
        })
    );
}

#[test]
fn the_command_line_takes_one_directory_and_any_links() -> TestResult {
    assert_eq!(args(&[]), Ok(LaunchRequest::default()));
    let parsed = args(&["work", "sp-harness://open?dir=x", "sp-harness://settings"])?;
    assert_eq!(parsed.directory, Some(PathBuf::from("work")));
    assert_eq!(parsed.links.len(), 2);
    assert_eq!(args(&["a", "b"]), Err(ArgsError::SecondDirectory));
    assert_eq!(
        args(&["--nope"]),
        Err(ArgsError::UnknownOption {
            option: "--nope".to_owned()
        })
    );
    Ok(())
}

#[test]
fn a_link_alone_names_the_project_to_open() -> TestResult {
    let request = args(&["sp-harness://settings", "sp-harness://open?dir=%2Fwork%2Fa"])?;
    assert_eq!(
        request.project_directory(SCHEME),
        Some(PathBuf::from("/work/a"))
    );
    let explicit = args(&["/work/b", "sp-harness://open?dir=%2Fwork%2Fa"])?;
    assert_eq!(
        explicit.project_directory(SCHEME),
        Some(PathBuf::from("/work/b"))
    );
    Ok(())
}

#[test]
fn requests_round_trip_as_json() -> TestResult {
    let request = LaunchRequest {
        directory: Some(PathBuf::from("/w")),
        links: vec!["sp-harness://settings".into()],
    };
    let text = serde_json::to_string(&request)?;
    assert_eq!(serde_json::from_str::<LaunchRequest>(&text)?, request);
    assert_eq!(
        serde_json::from_str::<LaunchRequest>("{}")?,
        LaunchRequest::default()
    );
    Ok(())
}
