//! The model and agent pickers the composer mounts.
//!
//! Each is a trigger showing the current choice that opens a searchable,
//! keyboard-driven list. Choosing goes through `AppState::set_model` /
//! `AppState::set_agent`, which remember the choice for the open project.
//!
//! Copyright (c) systemprompt.io — Business Source License 1.1.
//! See <https://systemprompt.io> for licensing details.

mod agent;
mod model;
mod popup;
mod popup_render;
pub mod search;

pub(crate) use agent::AgentPicker;
pub(crate) use model::ModelPicker;
