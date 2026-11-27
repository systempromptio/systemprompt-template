//! The UI model: GPUI entities kept current from the engine, store and accounts
//! feeds, over pure reducers that are tested without GPUI.
//!
//! Copyright (c) systemprompt.io — Business Source License 1.1.
//! See <https://systemprompt.io> for licensing details.

mod app_feeds;
mod app_intents;
mod app_navigation;
pub(crate) mod app_state;
pub(crate) mod extensions;
pub mod input;
pub(crate) mod loaders;
mod session_actions;
mod session_feed;
pub mod session_list;
pub mod session_model;
pub(crate) mod session_state;
pub(crate) mod transcript;

pub(crate) use app_state::{AppEvent, AppState, Notice, Route};
pub(crate) use input::Submission;
pub(crate) use session_list::{DayBucket, SessionRow};
pub(crate) use session_state::SessionState;
