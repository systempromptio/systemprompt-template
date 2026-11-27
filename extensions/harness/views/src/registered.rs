//! The named registration list of the shell extensions this build shows.
//!
//! Each entry is the shell half of one member of core's registration list
//! (`systemprompt_harness::extension::REGISTERED`) and is listed under that
//! member's own id constant, so the two lists pair by extension id.
//! Governance has no shell half here: its page is part of the frame.

use systemprompt_harness::members::brain::BRAIN_EXTENSION;
use systemprompt_harness::members::drive::DriveExtension;
use systemprompt_harness::members::notifications::NotificationsExtension;
use systemprompt_harness::members::slack::SlackExtension;
use systemprompt_harness_shared::contract::ShellExtensionRegistration;

use crate::brain::BrainShellExtension;
use crate::drive::DriveShellExtension;
use crate::notifications::{NotificationsShell, SlackShell};

pub const REGISTERED: [ShellExtensionRegistration; 4] = [
    ShellExtensionRegistration::of::<DriveShellExtension>(DriveExtension::ID),
    ShellExtensionRegistration::of::<BrainShellExtension>(BRAIN_EXTENSION),
    ShellExtensionRegistration::of::<NotificationsShell>(NotificationsExtension::ID),
    ShellExtensionRegistration::of::<SlackShell>(SlackExtension::ID),
];
