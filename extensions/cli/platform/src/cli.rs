//! Command-line surface: the clap types.
//!
//! Shape only. Every subcommand's behaviour lives in [`crate::commands`].

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name = "platform",
    about = "Grant, revoke and inspect platform-organization membership",
    long_about = "Platform admin is the `admin` role plus membership in the platform \
                  organization. `systemprompt admin users role promote` grants the role; \
                  these commands manage the membership."
)]
/// The parsed command line: one subcommand.
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

/// What to do with platform membership.
#[derive(Debug, Subcommand)]
pub enum Command {
    #[command(about = "Make an admin-role user a member of the platform organization")]
    Grant {
        #[arg(help = "The user's account name (usually their email)")]
        user: String,
    },
    #[command(about = "Remove a user's platform-organization membership")]
    Revoke {
        #[arg(help = "The user's account name (usually their email)")]
        user: String,
        #[arg(
            long,
            help = "Required to remove the last platform admin — after that nobody can \
                    assign elevated roles from the Admin UI until a grant runs again"
        )]
        force: bool,
    },
    #[command(about = "Show the platform organization and its members")]
    Status,
}
