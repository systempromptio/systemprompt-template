//! The English text of the Settings screen and its pages.

use systemprompt_engine::config::{Action, AgentMode};
use systemprompt_harness_brand::Channel;
use systemprompt_harness_shared::Scheme;

use crate::app::settings::DeliveryPref;

pub(crate) const OPEN_FAILED: &str = "The folder could not be opened.";
pub(crate) const OPEN_FOLDER: &str = "Open folder";
pub(crate) const REFRESH: &str = "Refresh";
pub(crate) const NO_PROJECT: &str = "Open a project to see its settings.";
pub(crate) const LOAD_FAILED: &str = "This page could not be loaded.";

pub(crate) const GENERAL: &str = "General";
pub(crate) const GENERAL_BODY: &str =
    "Appearance, what opens at start, and how the composer sends.";
pub(crate) const APPEARANCE: &str = "Appearance";
pub(crate) const SCHEME: &str = "Colour scheme";
pub(crate) const SCHEME_BODY: &str = "System follows the Windows light or dark setting.";
pub(crate) const STARTUP: &str = "Startup";
pub(crate) const START_FOLDER: &str = "Project opened at start";
pub(crate) const START_FOLDER_BODY: &str =
    "The app reopens the project you last worked in. Choose a folder to open next time instead.";
pub(crate) const CHOOSE_FOLDER: &str = "Choose…";
pub(crate) const CHOOSE_FOLDER_PROMPT: &str = "Project to open at start";
pub(crate) const USE_HOME: &str = "Use home folder";
pub(crate) const SHOW_SIDEBAR: &str = "Show the sidebar";
pub(crate) const SHOW_SIDEBAR_BODY: &str =
    "Also toggled with Ctrl+B; the choice is kept between starts.";
pub(crate) const COMPOSER: &str = "Composer";
pub(crate) const ENTER_SENDS: &str = "Send with Enter";
pub(crate) const ENTER_SENDS_BODY: &str = "When off, Enter adds a new line.";
pub(crate) const DELIVERY: &str = "Messages sent during a run";
pub(crate) const DELIVERY_BODY: &str =
    "Steer the agent right away, or queue the message until the run ends.";

pub(crate) const PERMISSIONS: &str = "Permissions";
pub(crate) const PERMISSIONS_BODY: &str = "The rules that decide whether a tool runs, asks first or is refused. Within \
     each group the last matching rule wins: built-in defaults come first, then your settings, the project's settings \
     and the agent's own rules.";
pub(crate) const RULES_FOR: &str = "Rules for agent";
pub(crate) const ADD_RULE: &str = "Add a rule";
pub(crate) const ADD: &str = "Add";
pub(crate) const PATTERN: &str = "Pattern, for example git *";
pub(crate) const REMOVE_RULE: &str = "Remove this rule";
pub(crate) const RULE_SAVE_FAILED: &str = "The rule could not be saved to your settings.";
pub(crate) const RULE_INVALID: &str = "Enter a pattern for the rule.";
pub(crate) const RULES_NOT_EDITABLE: &str =
    "Your settings file writes rules in a form this page cannot edit; change it in the file.";
pub(crate) const NO_RULES: &str = "No rules apply.";
pub(crate) const RULE_PROBLEM: &str = "A settings file could not be read; its rules are not shown:";
pub(crate) const SOURCE_DEFAULTS: &str = "Built-in defaults";
pub(crate) const SOURCE_USER: &str = "Your settings";
pub(crate) const SOURCE_PROJECT: &str = "Project settings";
pub(crate) const SOURCE_AGENT: &str = "Agent";
pub(crate) const ANY_PERMISSION: &str = "Every tool";

pub(crate) const AGENTS: &str = "Agents and skills";
pub(crate) const AGENTS_BODY: &str =
    "The agents and skills this project can use, and where each one is defined.";
pub(crate) const AGENT_LIST: &str = "Agents";
pub(crate) const SKILL_LIST: &str = "Skills";
pub(crate) const NO_AGENTS: &str = "No agents found.";
pub(crate) const NO_SKILLS: &str = "No skills found.";
pub(crate) const ORIGIN_BUILTIN: &str = "Built in";
pub(crate) const ORIGIN_SETTINGS: &str = "Defined in settings";
pub(crate) const SHOW_IN_FOLDER: &str = "Show in folder";

pub(crate) const MCP: &str = "MCP servers";
pub(crate) const MCP_BODY: &str = "Servers that give the agent extra tools. Servers your organisation manages are set by the instance and read-only.";
pub(crate) const NO_SERVERS: &str = "No MCP servers are configured for this project.";
pub(crate) const MANAGED: &str = "Managed";
pub(crate) const READ_ONLY: &str = "Read-only";
pub(crate) const OPEN_SETTINGS_FOLDER: &str = "Open settings folder";
pub(crate) const STATUS_CONNECTING: &str = "Connecting";
pub(crate) const STATUS_DISABLED: &str = "Disabled";
pub(crate) const STATUS_NEEDS_AUTH: &str = "Needs authorization";
pub(crate) const STATUS_FAILED: &str = "Failed";

pub(crate) const SHORTCUTS: &str = "Shortcuts";
pub(crate) const SHORTCUTS_BODY: &str = "Keyboard shortcuts. They are fixed in this version.";
pub(crate) const SHORTCUTS_APP: &str = "App";
pub(crate) const SHORTCUTS_LISTS: &str = "Lists and pickers";
pub(crate) const KEY_NEW_SESSION: &str = "New session";
pub(crate) const KEY_FOCUS_COMPOSER: &str = "Focus the composer";
pub(crate) const KEY_TOGGLE_SIDEBAR: &str = "Show or hide the sidebar";
pub(crate) const KEY_OPEN_SETTINGS: &str = "Open settings";
pub(crate) const KEY_STOP: &str = "Stop the run, or leave settings";
pub(crate) const KEY_PALETTE: &str = "Command palette";
pub(crate) const KEY_NEXT: &str = "Next item";
pub(crate) const KEY_PREVIOUS: &str = "Previous item";
pub(crate) const KEY_FIRST: &str = "First item";
pub(crate) const KEY_LAST: &str = "Last item";
pub(crate) const KEY_ACTIVATE: &str = "Open or choose";
pub(crate) const KEY_DISMISS: &str = "Close";

pub(crate) const ABOUT: &str = "About";
pub(crate) const ABOUT_BODY: &str = "This build, where it keeps its files, and its licence.";
pub(crate) const APP_NAME: &str = "App";
pub(crate) const VERSION: &str = "Version";
pub(crate) const CHANNEL: &str = "Channel";
pub(crate) const SIGNED_IN_AS: &str = "Signed in as";
pub(crate) const NOT_SIGNED_IN: &str = "Not signed in";
pub(crate) const DATA_FOLDER: &str = "Data folder";
pub(crate) const LOG_FOLDER: &str = "Log folder";
pub(crate) const LICENCES: &str = "Licences";
pub(crate) const LICENCES_BODY: &str = "The shell is licensed under the MIT License; the systemprompt.io core it runs is under the Business \
     Source License 1.1. Third-party components are \
     used under their own licences, listed with the installed app.";

pub(crate) fn with_detail(label: &str, detail: &str) -> String {
    format!("{label} · {detail}")
}

pub(crate) const fn scheme(scheme: Scheme) -> &'static str {
    match scheme {
        Scheme::System => "System",
        Scheme::Light => "Light",
        Scheme::Dark => "Dark",
    }
}

pub(crate) const fn delivery(delivery: DeliveryPref) -> &'static str {
    match delivery {
        DeliveryPref::Steer => "Steer the run",
        DeliveryPref::Queue => "Queue until it ends",
    }
}

pub(crate) const fn action(action: Action) -> &'static str {
    match action {
        Action::Allow => "Allow",
        Action::Ask => "Ask",
        Action::Deny => "Deny",
    }
}

pub(crate) const fn mode(mode: AgentMode) -> &'static str {
    match mode {
        AgentMode::Primary => "Primary",
        AgentMode::Subagent => "Subagent",
        AgentMode::All => "Primary and subagent",
    }
}

pub(crate) const fn channel(channel: Channel) -> &'static str {
    match channel {
        Channel::Internal => "Stable",
        Channel::Canary => "Canary",
        Channel::Dev => "Dev",
    }
}

pub(crate) fn connected(tools: usize) -> String {
    match tools {
        1 => "Connected, 1 tool".to_owned(),
        count => format!("Connected, {count} tools"),
    }
}

pub(crate) fn key_name(token: &str) -> Option<&'static str> {
    let name = match token {
        "secondary" => {
            if cfg!(target_os = "macos") {
                "Cmd"
            } else {
                "Ctrl"
            }
        },
        "ctrl" => "Ctrl",
        "cmd" => "Cmd",
        "shift" => "Shift",
        "alt" => "Alt",
        "escape" => "Esc",
        "enter" => "Enter",
        "up" => "Up",
        "down" => "Down",
        "home" => "Home",
        "end" => "End",
        "tab" => "Tab",
        "space" => "Space",
        _ => return None,
    };
    Some(name)
}
