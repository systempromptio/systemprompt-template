//! Handlebars rendering for the server-rendered admin pages.

mod engine;
pub mod helpers;

pub use engine::{AdminTemplateEngine, AdminTemplateError};
