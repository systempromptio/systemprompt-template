//! Actions-per-minute metrics derived from session event timing.

mod calculations;
mod queries;

pub use calculations::calculate_session_apm;
pub use queries::update_session_apm;
