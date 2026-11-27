//! Links into the app (`<scheme>://...`, the scheme being the build's own from
//! `systemprompt_harness_brand`) and the launch request a second start hands to
//! the running app.
//!
//! Links:
//!
//! - `<scheme>://open?dir=<path>` opens a project.
//! - `<scheme>://new?dir=<path>&prompt=<text>` opens a project (or the current
//!   one) on a new session with a draft.
//! - `<scheme>://session?id=<session id>` opens a session.
//! - `<scheme>://settings` or `<scheme>://settings?page=<id>` opens Settings.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use systemprompt_engine::store::{IdError, SessionId};
use url::Url;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Link {
    OpenProject {
        directory: PathBuf,
    },
    NewSession {
        directory: Option<PathBuf>,
        prompt: Option<String>,
    },
    OpenSession {
        session: SessionId,
    },
    OpenSettings {
        page: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum LinkError {
    #[error("not a URL")]
    Malformed(#[source] url::ParseError),
    #[error("the link is for scheme {found}, not this app")]
    ForeignScheme { found: String },
    #[error("unknown link action {action:?}")]
    UnknownAction { action: String },
    #[error("the link lacks its {name} parameter")]
    MissingParameter { name: &'static str },
    #[error("the link's session id is not valid")]
    InvalidSession(#[source] IdError),
}

pub fn is_link(scheme: &str, argument: &str) -> bool {
    argument
        .split_once("://")
        .is_some_and(|(found, _)| found.eq_ignore_ascii_case(scheme))
}

pub fn parse(scheme: &str, link: &str) -> Result<Link, LinkError> {
    let url = Url::parse(link).map_err(LinkError::Malformed)?;
    if !url.scheme().eq_ignore_ascii_case(scheme) {
        return Err(LinkError::ForeignScheme {
            found: url.scheme().to_owned(),
        });
    }
    let action = url.host_str().unwrap_or_default().to_ascii_lowercase();
    let action = if action.is_empty() {
        url.path().trim_matches('/').to_ascii_lowercase()
    } else {
        action
    };
    let query = |name: &str| {
        url.query_pairs()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.into_owned())
            .filter(|value| !value.trim().is_empty())
    };
    match action.as_str() {
        "open" => {
            let directory = query("dir").ok_or(LinkError::MissingParameter { name: "dir" })?;
            Ok(Link::OpenProject {
                directory: PathBuf::from(directory),
            })
        },
        "new" => Ok(Link::NewSession {
            directory: query("dir").map(PathBuf::from),
            prompt: query("prompt"),
        }),
        "session" => {
            let id = query("id").ok_or(LinkError::MissingParameter { name: "id" })?;
            let session = SessionId::parse(&id).map_err(LinkError::InvalidSession)?;
            Ok(Link::OpenSession { session })
        },
        "settings" => Ok(Link::OpenSettings {
            page: query("page"),
        }),
        other => Err(LinkError::UnknownAction {
            action: other.to_owned(),
        }),
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LaunchRequest {
    #[serde(default)]
    pub directory: Option<PathBuf>,
    #[serde(default)]
    pub links: Vec<String>,
}

impl LaunchRequest {
    pub fn project_directory(&self, scheme: &str) -> Option<PathBuf> {
        self.directory.clone().or_else(|| {
            self.links
                .iter()
                .find_map(|link| match parse(scheme, link) {
                    Ok(Link::OpenProject { directory }) => Some(directory),
                    Ok(Link::NewSession { directory, .. }) => directory,
                    _ => None,
                })
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ArgsError {
    #[error("unknown option {option}")]
    UnknownOption { option: String },
    #[error("only one directory can be opened")]
    SecondDirectory,
}

pub fn parse_args(
    scheme: &str,
    args: impl IntoIterator<Item = String>,
) -> Result<LaunchRequest, ArgsError> {
    let mut request = LaunchRequest::default();
    for argument in args {
        if is_link(scheme, &argument) {
            request.links.push(argument);
        } else if argument.starts_with("--") || (argument.starts_with('-') && argument.len() > 1) {
            return Err(ArgsError::UnknownOption { option: argument });
        } else if request.directory.is_some() {
            return Err(ArgsError::SecondDirectory);
        } else {
            request.directory = Some(PathBuf::from(argument));
        }
    }
    Ok(request)
}
