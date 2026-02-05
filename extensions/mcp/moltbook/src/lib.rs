#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::uninlined_format_args)]

mod server;
mod tools;

pub use server::MoltbookServer;
pub use tools::{list_tools, SERVER_NAME};
