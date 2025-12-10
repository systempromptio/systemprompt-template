mod artifact;
mod completion;
mod text;
mod tool_call;
mod tool_result;

pub use artifact::{handle_artifact_update, ArtifactHandleResult};
pub use completion::{handle_complete, handle_error, CompletionResult};
pub use text::handle_text;
pub use tool_call::handle_tool_call;
pub use tool_result::handle_tool_result;
