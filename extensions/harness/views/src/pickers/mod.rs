//! The model and agent pickers the composer mounts.
//!
//! Each is a trigger showing the current choice that opens a searchable,
//! keyboard-driven list. Choosing goes through `AppState::set_model` /
//! `AppState::set_agent`, which remember the choice for the open project.

mod agent;
mod model;
mod popup;
mod popup_render;
pub mod search;

pub(crate) use agent::AgentPicker;
pub(crate) use model::ModelPicker;
