//! The Brain's GPUI views: the screen, the settings pane and the transcript
//! adornment, with the bridge that runs the brain's calls on tokio.

mod adornment;
mod bridge;
mod screen;
mod settings;
mod tone;

pub(crate) use adornment::RecalledFacts;
pub(crate) use screen::BrainScreen;
pub(crate) use settings::BrainSettingsPane;
